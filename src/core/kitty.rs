use std::collections::HashMap;

use base64::{Engine as _, engine::general_purpose::STANDARD};

const MAX_PENDING_GRAPHICS_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, Default)]
pub struct KittyGraphicsTracker {
    pending: Vec<u8>,
}

impl KittyGraphicsTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, bytes: &[u8]) -> Vec<KittyGraphicsCommand> {
        self.pending.extend_from_slice(bytes);
        let mut commands = Vec::new();

        loop {
            let Some(start) = find_apc_graphics_start(&self.pending) else {
                trim_apc_pending(&mut self.pending);
                return commands;
            };

            if start > 0 {
                self.pending.drain(..start);
            }

            let payload_start = if self.pending.starts_with(b"\x1b_G") {
                3
            } else {
                2
            };
            let Some((end, terminator_len)) = find_apc_end(&self.pending, payload_start) else {
                if self.pending.len() > MAX_PENDING_GRAPHICS_BYTES {
                    let keep_from = self
                        .pending
                        .len()
                        .saturating_sub(MAX_PENDING_GRAPHICS_BYTES);
                    self.pending.drain(..keep_from);
                }
                return commands;
            };

            if let Some(command) = KittyGraphicsCommand::parse(&self.pending[payload_start..end]) {
                commands.push(command);
            }
            self.pending.drain(..end + terminator_len);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyGraphicsCommand {
    pub action: KittyGraphicsAction,
    pub image_id: Option<u32>,
    pub placement_id: Option<u32>,
    pub format: Option<KittyImageFormat>,
    pub transmission: Option<KittyTransmission>,
    pub more_chunks: bool,
    pub quiet: Option<u8>,
    pub columns: Option<u32>,
    pub rows: Option<u32>,
    pub z_index: Option<i32>,
    pub decoded_payload: Vec<u8>,
    pub raw_payload: Vec<u8>,
}

impl KittyGraphicsCommand {
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let (control, payload) = bytes
            .iter()
            .position(|byte| *byte == b';')
            .map_or((bytes, [].as_slice()), |separator| {
                (&bytes[..separator], &bytes[separator + 1..])
            });
        let control_text = std::str::from_utf8(control).ok()?;
        let mut action = KittyGraphicsAction::Transmit;
        let mut image_id = None;
        let mut placement_id = None;
        let mut format = None;
        let mut transmission = None;
        let mut more_chunks = false;
        let mut quiet = None;
        let mut columns = None;
        let mut rows = None;
        let mut z_index = None;

        for item in control_text.split(',').filter(|item| !item.is_empty()) {
            let Some((key, value)) = item.split_once('=') else {
                continue;
            };
            match key {
                "a" => action = KittyGraphicsAction::from_value(value),
                "i" => image_id = parse_u32(value),
                "p" => placement_id = parse_u32(value),
                "f" => format = Some(KittyImageFormat::from_value(value)),
                "t" => transmission = Some(KittyTransmission::from_value(value)),
                "m" => more_chunks = value == "1",
                "q" => quiet = parse_u8(value),
                "c" => columns = parse_u32(value),
                "r" => rows = parse_u32(value),
                "z" => z_index = value.parse::<i32>().ok(),
                _ => {}
            }
        }

        let decoded_payload = STANDARD.decode(payload).unwrap_or_default();
        Some(Self {
            action,
            image_id,
            placement_id,
            format,
            transmission,
            more_chunks,
            quiet,
            columns,
            rows,
            z_index,
            decoded_payload,
            raw_payload: payload.to_vec(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyGraphicsAction {
    Transmit,
    TransmitAndDisplay,
    Display,
    Delete,
    Query,
    Unknown(char),
}

impl KittyGraphicsAction {
    fn from_value(value: &str) -> Self {
        match value {
            "t" => Self::Transmit,
            "T" => Self::TransmitAndDisplay,
            "p" => Self::Display,
            "d" => Self::Delete,
            "q" => Self::Query,
            _ => value.chars().next().map_or(Self::Transmit, Self::Unknown),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyImageFormat {
    Rgb,
    Rgba,
    Png,
    Unknown(u32),
}

impl KittyImageFormat {
    fn from_value(value: &str) -> Self {
        match parse_u32(value) {
            Some(24) => Self::Rgb,
            Some(32) => Self::Rgba,
            Some(100) => Self::Png,
            Some(value) => Self::Unknown(value),
            None => Self::Unknown(0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyTransmission {
    Direct,
    File,
    TemporaryFile,
    SharedMemory,
    Unknown(char),
}

impl KittyTransmission {
    fn from_value(value: &str) -> Self {
        match value {
            "d" => Self::Direct,
            "f" => Self::File,
            "t" => Self::TemporaryFile,
            "s" => Self::SharedMemory,
            _ => value.chars().next().map_or(Self::Direct, Self::Unknown),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct KittyGraphicsState {
    images: HashMap<u32, KittyImageResource>,
    placements: HashMap<KittyPlacementKey, KittyImagePlacement>,
}

impl KittyGraphicsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, command: KittyGraphicsCommand) -> Vec<KittyGraphicsEvent> {
        match command.action {
            KittyGraphicsAction::Delete => self.delete(command.image_id, command.placement_id),
            KittyGraphicsAction::Transmit
            | KittyGraphicsAction::TransmitAndDisplay
            | KittyGraphicsAction::Display => self.upsert(command),
            KittyGraphicsAction::Query | KittyGraphicsAction::Unknown(_) => {
                vec![KittyGraphicsEvent::Unsupported]
            }
        }
    }

    pub fn image(&self, id: u32) -> Option<&KittyImageResource> {
        self.images.get(&id)
    }

    pub fn placements(&self) -> impl Iterator<Item = &KittyImagePlacement> {
        self.placements.values()
    }

    fn upsert(&mut self, command: KittyGraphicsCommand) -> Vec<KittyGraphicsEvent> {
        let Some(image_id) = command.image_id else {
            return vec![KittyGraphicsEvent::Unsupported];
        };
        let mut events = Vec::new();

        if command.action != KittyGraphicsAction::Display {
            let resource = self
                .images
                .entry(image_id)
                .or_insert_with(|| KittyImageResource::new(image_id));
            if let Some(format) = command.format {
                resource.format = Some(format);
            }
            if let Some(transmission) = command.transmission {
                resource.transmission = Some(transmission);
            }
            resource.bytes.extend_from_slice(&command.decoded_payload);
            resource.complete = !command.more_chunks;
            events.push(KittyGraphicsEvent::ImageUpdated(image_id));
        }

        if matches!(
            command.action,
            KittyGraphicsAction::TransmitAndDisplay | KittyGraphicsAction::Display
        ) {
            let key = KittyPlacementKey {
                image_id,
                placement_id: command.placement_id.unwrap_or(0),
            };
            self.placements.insert(
                key,
                KittyImagePlacement {
                    key,
                    columns: command.columns,
                    rows: command.rows,
                    z_index: command.z_index.unwrap_or(0),
                },
            );
            events.push(KittyGraphicsEvent::PlacementUpdated(key));
        }

        events
    }

    fn delete(
        &mut self,
        image_id: Option<u32>,
        placement_id: Option<u32>,
    ) -> Vec<KittyGraphicsEvent> {
        let Some(image_id) = image_id else {
            self.images.clear();
            self.placements.clear();
            return vec![KittyGraphicsEvent::Deleted {
                image_id: None,
                placement_id: None,
            }];
        };

        if let Some(placement_id) = placement_id {
            self.placements.remove(&KittyPlacementKey {
                image_id,
                placement_id,
            });
        } else {
            self.images.remove(&image_id);
            self.placements.retain(|key, _| key.image_id != image_id);
        }

        vec![KittyGraphicsEvent::Deleted {
            image_id: Some(image_id),
            placement_id,
        }]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyImageResource {
    pub id: u32,
    pub format: Option<KittyImageFormat>,
    pub transmission: Option<KittyTransmission>,
    pub complete: bool,
    pub bytes: Vec<u8>,
}

impl KittyImageResource {
    fn new(id: u32) -> Self {
        Self {
            id,
            format: None,
            transmission: None,
            complete: false,
            bytes: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct KittyPlacementKey {
    pub image_id: u32,
    pub placement_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyImagePlacement {
    pub key: KittyPlacementKey,
    pub columns: Option<u32>,
    pub rows: Option<u32>,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KittyGraphicsEvent {
    ImageUpdated(u32),
    PlacementUpdated(KittyPlacementKey),
    Deleted {
        image_id: Option<u32>,
        placement_id: Option<u32>,
    },
    Unsupported,
}

fn find_apc_graphics_start(bytes: &[u8]) -> Option<usize> {
    let esc = bytes.windows(3).position(|window| window == b"\x1b_G");
    let c1 = bytes.windows(2).position(|window| window == [0x9f, b'G']);

    match (esc, c1) {
        (Some(esc), Some(c1)) => Some(esc.min(c1)),
        (Some(esc), None) => Some(esc),
        (None, Some(c1)) => Some(c1),
        (None, None) => None,
    }
}

fn find_apc_end(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut idx = start;
    while idx < bytes.len() {
        if bytes[idx] == 0x9c {
            return Some((idx, 1));
        }
        if idx + 1 < bytes.len() && bytes[idx] == 0x1b && bytes[idx + 1] == b'\\' {
            return Some((idx, 2));
        }
        idx += 1;
    }
    None
}

fn trim_apc_pending(pending: &mut Vec<u8>) {
    if pending.ends_with(b"\x1b_") || pending.ends_with(&[0x9f]) {
        let keep_from = pending.len().saturating_sub(2);
        pending.drain(..keep_from);
    } else if pending.last() == Some(&0x1b) {
        let Some(esc) = pending.pop() else {
            return;
        };
        pending.clear();
        pending.push(esc);
    } else {
        pending.clear();
    }
}

fn parse_u32(value: &str) -> Option<u32> {
    value.parse::<u32>().ok()
}

fn parse_u8(value: &str) -> Option<u8> {
    value.parse::<u8>().ok()
}

#[cfg(test)]
mod tests {
    use super::{
        KittyGraphicsAction, KittyGraphicsEvent, KittyGraphicsState, KittyGraphicsTracker,
        KittyImageFormat, KittyPlacementKey, KittyTransmission,
    };

    #[test]
    fn tracker_parses_split_direct_png_command() {
        let mut tracker = KittyGraphicsTracker::new();

        assert!(tracker.push(b"\x1b_Ga=T,f=100,t=d,i=42,p=7,c=4").is_empty());
        let commands = tracker.push(b",r=2,z=3;YWJj\x1b\\tail");

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].action, KittyGraphicsAction::TransmitAndDisplay);
        assert_eq!(commands[0].format, Some(KittyImageFormat::Png));
        assert_eq!(commands[0].transmission, Some(KittyTransmission::Direct));
        assert_eq!(commands[0].image_id, Some(42));
        assert_eq!(commands[0].placement_id, Some(7));
        assert_eq!(commands[0].columns, Some(4));
        assert_eq!(commands[0].rows, Some(2));
        assert_eq!(commands[0].z_index, Some(3));
        assert_eq!(commands[0].decoded_payload, b"abc");
    }

    #[test]
    fn state_tracks_resource_and_placement() {
        let command =
            KittyGraphicsTracker::new().push(b"\x1b_Ga=T,f=100,t=d,i=9,p=1,c=8,r=3;YWJj\x1b\\");
        let mut state = KittyGraphicsState::new();

        let events = state.apply(command[0].clone());

        assert_eq!(
            events,
            vec![
                KittyGraphicsEvent::ImageUpdated(9),
                KittyGraphicsEvent::PlacementUpdated(KittyPlacementKey {
                    image_id: 9,
                    placement_id: 1,
                }),
            ]
        );
        let image = state.image(9).expect("image should be stored");
        assert_eq!(image.bytes, b"abc");
        assert!(image.complete);
        assert_eq!(state.placements().count(), 1);
    }

    #[test]
    fn state_deletes_image_and_placements() {
        let mut tracker = KittyGraphicsTracker::new();
        let mut state = KittyGraphicsState::new();
        let command = tracker.push(b"\x1b_Ga=T,i=9,p=1;YQ==\x1b\\");
        state.apply(command[0].clone());

        let delete = tracker.push(b"\x1b_Ga=d,i=9\x1b\\");
        let events = state.apply(delete[0].clone());

        assert_eq!(
            events,
            vec![KittyGraphicsEvent::Deleted {
                image_id: Some(9),
                placement_id: None,
            }]
        );
        assert!(state.image(9).is_none());
        assert_eq!(state.placements().count(), 0);
    }
}

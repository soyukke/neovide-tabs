import CryptoKit
import Foundation

struct SigningOutput: Encodable {
    let algorithm: String
    let publicKey: String
    let signature: String
}

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data("sign-update: \(message)\n".utf8))
    exit(64)
}

guard CommandLine.arguments.count == 2 else {
    fail("usage: sign-update.swift ARCHIVE")
}
guard let encodedKey = ProcessInfo.processInfo.environment[
    "UPDATE_SIGNING_PRIVATE_KEY_B64"
],
let rawKey = Data(base64Encoded: encodedKey),
rawKey.count == 32
else {
    fail("UPDATE_SIGNING_PRIVATE_KEY_B64 must contain a base64-encoded 32-byte key")
}

do {
    let archiveURL = URL(fileURLWithPath: CommandLine.arguments[1])
    let archive = try Data(contentsOf: archiveURL, options: .mappedIfSafe)
    let privateKey = try Curve25519.Signing.PrivateKey(rawRepresentation: rawKey)
    let signature = try privateKey.signature(for: archive)
    let output = SigningOutput(
        algorithm: "ed25519",
        publicKey: privateKey.publicKey.rawRepresentation.base64EncodedString(),
        signature: signature.base64EncodedString()
    )
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.sortedKeys]
    FileHandle.standardOutput.write(try encoder.encode(output))
    FileHandle.standardOutput.write(Data("\n".utf8))
} catch {
    fail(error.localizedDescription)
}

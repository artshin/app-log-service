import Foundation

/// Provides a persistent device identifier stored in UserDefaults.
///
/// Generates a UUID on first use and persists it across app launches.
public enum DeviceIdentifier: Sendable {
    private static let key = "AppLogService.deviceId"

    /// The persistent device identifier. Generated on first access.
    public static var id: String {
        if let existing = UserDefaults.standard.string(forKey: key) {
            return existing
        }
        let newId = UUID().uuidString
        UserDefaults.standard.set(newId, forKey: key)
        return newId
    }

    /// Resets the device identifier. A new one will be generated on next access.
    public static func reset() {
        UserDefaults.standard.removeObject(forKey: key)
    }
}

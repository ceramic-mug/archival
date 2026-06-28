import SwiftUI
import ArchivalUI

@main
struct ArchivalEntry: App {
    var body: some Scene {
        WindowGroup {
            RootView()
        }
        #if os(macOS)
        .defaultSize(width: 1100, height: 700)
        #endif
        #if os(macOS)
        Settings {
            SettingsView()
        }
        #endif
    }
}

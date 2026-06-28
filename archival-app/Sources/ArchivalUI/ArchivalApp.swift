import SwiftUI
import ArchivalCore

// @main lives in the Xcode app target entry file, not here.
// This file exports the root scene body so app targets can reference it.

@MainActor
public final class AppState: ObservableObject {
    @Published public var db: ArchivalDB?
    @Published public var error: String?
    @AppStorage("geminiAPIKey") public var apiKey: String = ""

    public init() {}

    public func open() async {
        let (dbPath, mediaRoot) = ArchivalDB.defaultPaths()
        do {
            db = try ArchivalDB(dbPath: dbPath, mediaRoot: mediaRoot)
        } catch {
            self.error = error.localizedDescription
        }
    }
}

public struct RootView: View {
    @StateObject private var appState = AppState()

    public init() {}

    public var body: some View {
        Group {
            if let db = appState.db {
                ContentView(db: db, apiKey: appState.apiKey)
            } else if let error = appState.error {
                ErrorView(message: error)
            } else {
                ProgressView("Opening archive…")
            }
        }
        .task { await appState.open() }
    }
}

public struct ErrorView: View {
    public let message: String

    public init(message: String) { self.message = message }

    public var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundStyle(.red)
            Text("Failed to open archive")
                .font(.headline)
            Text(message)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding()
    }
}

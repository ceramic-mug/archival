import SwiftUI
import ArchivalCore

public struct SettingsView: View {
    @AppStorage("geminiAPIKey") private var apiKey: String = ""

    private var dbPath: String {
        ArchivalDB.defaultPaths().dbPath
    }

    public init() {}

    public var body: some View {
        Form {
            Section {
                LabeledContent("Gemini API Key") {
                    SecureField("Paste your key here", text: $apiKey)
                        .textFieldStyle(.roundedBorder)
                        .frame(minWidth: 260)
                }
            } header: {
                Text("AI Identification")
            } footer: {
                Text("Used to classify photos and fill item fields. Stored in app preferences.")
                    .foregroundStyle(.secondary)
            }

            Section("Archive Location") {
                LabeledContent("Database") {
                    Text(dbPath)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                }
                Button("Reveal in Finder") {
                    NSWorkspace.shared.selectFile(dbPath, inFileViewerRootedAtPath: "")
                }
            }
        }
        .formStyle(.grouped)
        .frame(width: 480)
        .padding()
    }
}

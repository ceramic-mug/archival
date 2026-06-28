import SwiftUI
import ArchivalCore

public struct ItemDetailView: View {
    @ObservedObject var db: ArchivalDB
    let itemId: String

    @State private var fields: [String: String?] = [:]
    @State private var tags: [String] = []
    @State private var media: [ArchivalMediaFile] = []
    @State private var isEditing = false
    @State private var editedFields: [String: String] = [:]
    @State private var errorMessage: String?

    public var body: some View {
        Form {
            Section("Fields") {
                if fields.isEmpty {
                    Text("No fields loaded")
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(Array(fields.sorted(by: { $0.key < $1.key })), id: \.key) { key, value in
                        if isEditing {
                            LabeledContent(key) {
                                TextField("", text: Binding(
                                    get: { editedFields[key] ?? value ?? "" },
                                    set: { editedFields[key] = $0 }
                                ))
                                .multilineTextAlignment(.trailing)
                            }
                        } else {
                            LabeledContent(key, value: value ?? "—")
                        }
                    }
                }
            }

            if !tags.isEmpty {
                Section("Tags") {
                    FlowLayout(tags: tags)
                }
            }

            if !media.isEmpty {
                Section("Media") {
                    ForEach(media) { file in
                        Text(file.filePath)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .navigationTitle(itemId.prefix(8) + "…")
        .toolbar {
            if isEditing {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Save") { saveFields() }
                }
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { isEditing = false; editedFields = [:] }
                }
            } else {
                ToolbarItem(placement: .primaryAction) {
                    Button("Edit") { isEditing = true }
                }
            }
        }
        .task { loadAll() }
        .alert("Error", isPresented: Binding(get: { errorMessage != nil }, set: { if !$0 { errorMessage = nil } })) {
            Button("OK") { errorMessage = nil }
        } message: {
            Text(errorMessage ?? "")
        }
    }

    private func loadAll() {
        do {
            fields = try db.getFields(itemId: itemId)
            tags   = try db.tags(itemId: itemId)
            media  = try db.listMedia(itemId: itemId)
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func saveFields() {
        let merged: [String: String?] = editedFields.mapValues { Optional($0) }
        do {
            try db.setFields(itemId: itemId, fields: merged)
            loadAll()
            isEditing = false
            editedFields = [:]
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

struct FlowLayout: View {
    let tags: [String]

    public var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack {
                ForEach(tags, id: \.self) { tag in
                    Text(tag)
                        .font(.caption)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(.quaternary, in: Capsule())
                }
            }
        }
    }
}

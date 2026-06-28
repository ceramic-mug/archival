import SwiftUI
import PhotosUI
import ArchivalCore

public struct AddItemView: View {
    @ObservedObject var db: ArchivalDB
    let apiKey: String
    let onComplete: (String) -> Void

    @State private var addMode: AddMode = .photo
    @State private var selectedPhoto: PhotosPickerItem?
    @State private var imageData: Data?
    @State private var phase: AddPhase = .pick
    @State private var pendingItemId: String?
    @State private var pendingCategory: ArchivalCategory = .object
    @State private var pendingFields: [String: String?] = [:]
    @State private var editedFields: [String: String] = [:]
    @State private var isProcessing = false
    @State private var processingStep: String = ""
    @State private var errorMessage: String?

    enum AddMode: String, CaseIterable {
        case photo = "Photo"
        case manual = "Manual"
    }

    enum AddPhase { case pick, review, done }

    public var body: some View {
        NavigationStack {
            Group {
                switch phase {
                case .pick:
                    pickView
                case .review:
                    reviewView
                case .done:
                    Text("Item added!")
                        .font(.headline)
                        .task { onComplete(pendingItemId ?? "") }
                }
            }
            .navigationTitle("Add Item")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { onComplete("") }
                }
            }
            .alert("Error", isPresented: Binding(get: { errorMessage != nil }, set: { if !$0 { errorMessage = nil } })) {
                Button("OK") { errorMessage = nil }
            } message: {
                Text(errorMessage ?? "")
            }
        }
    }

    // MARK: Step 1 — Pick (Photo or Manual)

    private var pickView: some View {
        VStack(spacing: 24) {
            Picker("", selection: $addMode) {
                ForEach(AddMode.allCases, id: \.self) { mode in
                    Text(mode.rawValue).tag(mode)
                }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)

            if addMode == .photo {
                photoPickSection
            } else {
                manualPickSection
            }
        }
        .padding()
    }

    private var photoPickSection: some View {
        VStack(spacing: 20) {
            if let data = imageData {
                #if os(macOS)
                if let ns = NSImage(data: data) {
                    Image(nsImage: ns)
                        .resizable()
                        .scaledToFit()
                        .frame(maxHeight: 280)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                }
                #else
                if let ui = UIImage(data: data) {
                    Image(uiImage: ui)
                        .resizable()
                        .scaledToFit()
                        .frame(maxHeight: 280)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                }
                #endif
            } else {
                RoundedRectangle(cornerRadius: 12)
                    .fill(.quaternary)
                    .frame(height: 200)
                    .overlay {
                        Image(systemName: "photo")
                            .font(.largeTitle)
                            .foregroundStyle(.secondary)
                    }
            }

            PhotosPicker("Select Photo", selection: $selectedPhoto, matching: .images)
                .buttonStyle(.borderedProminent)

            if imageData != nil {
                if isProcessing {
                    processingView
                } else {
                    Button("Identify & Catalogue") {
                        Task { await classifyPhoto() }
                    }
                    .buttonStyle(.bordered)
                    .disabled(apiKey.isEmpty)

                    if apiKey.isEmpty {
                        Text("Set your Gemini API key in Settings (⌘,) first.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .onChange(of: selectedPhoto) { _, item in
            Task {
                imageData = try? await item?.loadTransferable(type: Data.self)
            }
        }
    }

    private var manualPickSection: some View {
        VStack(spacing: 20) {
            Form {
                Picker("Category", selection: $pendingCategory) {
                    ForEach(ArchivalCategory.allCases) { cat in
                        Label(cat.displayName, systemImage: cat.sfSymbol).tag(cat)
                    }
                }
            }
            .frame(height: 60)

            Button("Catalogue by Hand") {
                Task { await addManually() }
            }
            .buttonStyle(.borderedProminent)
            .disabled(isProcessing)
        }
    }

    private var processingView: some View {
        VStack(spacing: 16) {
            ZStack {
                Circle()
                    .stroke(.quaternary, lineWidth: 4)
                    .frame(width: 48, height: 48)
                SpinningArc()
                    .frame(width: 48, height: 48)
            }
            Text(processingStep)
                .font(.callout)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .animation(.default, value: processingStep)
        }
        .padding()
    }

    // MARK: Step 2 — Review

    private var reviewView: some View {
        Form {
            Section("Category") {
                Picker("Category", selection: $pendingCategory) {
                    ForEach(ArchivalCategory.allCases) { cat in
                        Text(cat.displayName).tag(cat)
                    }
                }
            }
            Section("Fields") {
                ForEach(Array(pendingFields.sorted(by: { $0.key < $1.key })), id: \.key) { key, value in
                    LabeledContent(key) {
                        TextField("", text: Binding(
                            get: { editedFields[key] ?? value ?? "" },
                            set: { editedFields[key] = $0 }
                        ))
                        .multilineTextAlignment(.trailing)
                    }
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Save") {
                    Task { await confirm() }
                }
            }
        }
    }

    // MARK: - Actions

    private func classifyPhoto() async {
        guard let data = imageData else { return }
        isProcessing = true
        defer { isProcessing = false }

        do {
            let tmp = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".jpg")
            try data.write(to: tmp)
            defer { try? FileManager.default.removeItem(at: tmp) }

            processingStep = "Analyzing photo…"
            let categoryName = try db.classifyImage(imagePath: tmp.path, apiKey: apiKey)
            let category = ArchivalCategory(rawValue: categoryName) ?? .object
            processingStep = "Found a \(category.displayName) — cataloguing fields…"

            let itemId = try db.createItem(category: category)
            let fields = try db.fillFields(itemId: itemId, imagePath: tmp.path, apiKey: apiKey)
            processingStep = "Almost done…"

            pendingItemId   = itemId
            pendingCategory = category
            pendingFields   = fields
            phase = .review
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func addManually() async {
        isProcessing = true
        defer { isProcessing = false }
        do {
            let itemId = try db.createItem(category: pendingCategory)
            let fields = try db.getFields(itemId: itemId)
            pendingItemId   = itemId
            pendingFields   = fields
            phase = .review
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func confirm() async {
        guard let itemId = pendingItemId else { return }
        do {
            let merged: [String: String?] = editedFields.mapValues { Optional($0) }
            let final = pendingFields.merging(merged) { _, new in new }
            try db.setFields(itemId: itemId, fields: final)
            phase = .done
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

// MARK: - Spinning Arc Animation

private struct SpinningArc: View {
    @State private var rotation: Double = 0

    var body: some View {
        Circle()
            .trim(from: 0.1, to: 0.85)
            .stroke(
                AngularGradient(
                    colors: [.accentColor.opacity(0.2), .accentColor],
                    center: .center
                ),
                style: StrokeStyle(lineWidth: 4, lineCap: .round)
            )
            .rotationEffect(.degrees(rotation))
            .onAppear {
                withAnimation(.linear(duration: 1).repeatForever(autoreverses: false)) {
                    rotation = 360
                }
            }
    }
}

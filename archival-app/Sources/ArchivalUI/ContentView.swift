import SwiftUI
import ArchivalCore

public struct ContentView: View {
    @ObservedObject var db: ArchivalDB
    let apiKey: String
    @State private var selectedCategory: ArchivalCategory?

    public init(db: ArchivalDB, apiKey: String) {
        self.db = db
        self.apiKey = apiKey
    }

    public var body: some View {
        NavigationSplitView {
            CategorySidebar(selectedCategory: $selectedCategory)
        } detail: {
            NavigationStack {
                ItemListView(db: db, apiKey: apiKey, category: selectedCategory)
            }
        }
    }
}

struct CategorySidebar: View {
    @Binding var selectedCategory: ArchivalCategory?

    var body: some View {
        List(selection: $selectedCategory) {
            Section("Browse") {
                Label("All Items", systemImage: "tray.full")
                    .tag(Optional<ArchivalCategory>.none)
            }
            Section("Categories") {
                ForEach(ArchivalCategory.allCases) { cat in
                    Label(cat.displayName, systemImage: cat.sfSymbol)
                        .tag(Optional(cat))
                }
            }
        }
        #if os(macOS)
        .listStyle(.sidebar)
        #endif
        .navigationTitle("Archival")
    }
}

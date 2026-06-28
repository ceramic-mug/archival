import SwiftUI
import ArchivalCore

public struct ItemListView: View {
    @ObservedObject var db: ArchivalDB
    let apiKey: String
    let category: ArchivalCategory?

    @State private var items: [ArchivalItemSummary] = []
    @State private var searchText = ""
    @State private var showingAddItem = false
    @State private var errorMessage: String?
    @State private var selectedItemId: String?

    var filteredItems: [ArchivalItemSummary] {
        guard !searchText.isEmpty else { return items }
        return items.filter {
            $0.listTitle.localizedCaseInsensitiveContains(searchText) ||
            $0.category.localizedCaseInsensitiveContains(searchText) ||
            ($0.notes?.localizedCaseInsensitiveContains(searchText) ?? false)
        }
    }

    public var body: some View {
        Group {
            if items.isEmpty {
                ContentUnavailableView(
                    "No items yet",
                    systemImage: category?.sfSymbol ?? "tray",
                    description: Text("Tap + to add your first item.")
                )
            } else {
                List(filteredItems, selection: $selectedItemId) { item in
                    NavigationLink(value: item.id) {
                        ItemRow(item: item)
                    }
                }
                .navigationDestination(for: String.self) { itemId in
                    ItemDetailView(db: db, itemId: itemId)
                }
            }
        }
        .searchable(text: $searchText)
        .navigationTitle(category?.displayName ?? "All Items")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button { showingAddItem = true } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showingAddItem) {
            AddItemView(db: db, apiKey: apiKey) { _ in
                loadItems()
                showingAddItem = false
            }
        }
        .task(id: category) { loadItems() }
        .alert("Error", isPresented: Binding(get: { errorMessage != nil }, set: { if !$0 { errorMessage = nil } })) {
            Button("OK") { errorMessage = nil }
        } message: {
            Text(errorMessage ?? "")
        }
    }

    private func loadItems() {
        do {
            items = try db.listItems(category: category)
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

struct ItemRow: View {
    let item: ArchivalItemSummary

    public var body: some View {
        HStack {
            Image(systemName: item.archivalCategory?.sfSymbol ?? "cube")
                .frame(width: 28)
                .foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 2) {
                Text(item.listTitle)
                    .font(.headline)
                Text(item.archivalCategory?.displayName ?? item.category)
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            }
            Spacer()
            Text(Date(timeIntervalSince1970: Double(item.createdAt)), style: .date)
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
    }
}

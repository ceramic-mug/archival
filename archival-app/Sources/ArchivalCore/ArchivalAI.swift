import Foundation

/// Drives the two-step AI ingestion pipeline: classify → fill fields.
/// Results are written directly to the DB via ArchivalDB.
@MainActor
public struct ArchivalAI {
    public let db: ArchivalDB
    public let apiKey: String

    public init(db: ArchivalDB, apiKey: String) {
        self.db = db
        self.apiKey = apiKey
    }

    /// Full pipeline: classify image, create item, fill fields, return item id + suggested fields.
    public func ingest(imagePath: String) async throws -> (itemId: String, category: ArchivalCategory, fields: [String: String?]) {
        let categoryName = try db.classifyImage(imagePath: imagePath, apiKey: apiKey)
        let category = ArchivalCategory(rawValue: categoryName) ?? .object

        let itemId = try db.createItem(category: category)
        let fields = try db.fillFields(itemId: itemId, imagePath: imagePath, apiKey: apiKey)

        return (itemId, category, fields)
    }

    /// Confirm: write user-reviewed fields to DB.
    public func confirmFields(itemId: String, fields: [String: String?]) throws {
        try db.setFields(itemId: itemId, fields: fields)
    }
}

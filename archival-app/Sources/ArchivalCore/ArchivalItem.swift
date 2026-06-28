import Foundation

public enum ArchivalCategory: String, Codable, CaseIterable, Identifiable {
    case book            = "Book"
    case music           = "Music"
    case movie           = "Movie"
    case game            = "Game"
    case personalMessage = "PersonalMessage"
    case award           = "Award"
    case art             = "Art"
    case photograph      = "Photograph"
    case trinket         = "Trinket"
    case jewelry         = "Jewelry"
    case clothing        = "Clothing"
    case object          = "Object"

    public var id: String { rawValue }

    public var displayName: String {
        switch self {
        case .personalMessage: return "Personal Message"
        default: return rawValue
        }
    }

    public var specialistName: String {
        switch self {
        case .book:            return "Librarian"
        case .music:           return "DJ"
        case .movie:           return "Critic"
        case .game:            return "Archivist"
        case .personalMessage: return "Correspondent"
        case .award:           return "Curator"
        case .art:             return "Appraiser"
        case .photograph:      return "Archivist"
        case .trinket:         return "Curator"
        case .jewelry:         return "Appraiser"
        case .clothing:        return "Curator"
        case .object:          return "Archivist"
        }
    }

    public var sfSymbol: String {
        switch self {
        case .book:            return "book.closed"
        case .music:           return "music.note"
        case .movie:           return "film"
        case .game:            return "gamecontroller"
        case .personalMessage: return "envelope"
        case .award:           return "trophy"
        case .art:             return "paintbrush"
        case .photograph:      return "photo"
        case .trinket:         return "sparkles"
        case .jewelry:         return "diamond"
        case .clothing:        return "tshirt"
        case .object:          return "cube"
        }
    }
}

public struct ArchivalItemSummary: Identifiable, Codable {
    public let id: String
    public let category: String
    public let createdAt: Int
    public let updatedAt: Int
    public let notes: String?
    public let displayName: String?

    public var archivalCategory: ArchivalCategory? {
        ArchivalCategory(rawValue: category)
    }

    public var listTitle: String {
        displayName ?? archivalCategory?.displayName ?? category
    }

    enum CodingKeys: String, CodingKey {
        case id, category
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case notes
        case displayName = "display_name"
    }
}

public struct ArchivalMediaFile: Identifiable, Codable {
    public let id: String
    public let filePath: String
    public let mimeType: String
    public let isPrimary: Int
    public let createdAt: Int

    enum CodingKeys: String, CodingKey {
        case id
        case filePath = "file_path"
        case mimeType = "mime_type"
        case isPrimary = "is_primary"
        case createdAt = "created_at"
    }
}

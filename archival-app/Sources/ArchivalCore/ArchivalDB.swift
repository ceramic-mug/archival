import Foundation
import CArchivalCore

/// Owns the Rust DbHandle pointer. Confined to @MainActor — pass async calls through a Task.
@MainActor
public final class ArchivalDB: ObservableObject {
    private var handle: OpaquePointer?

    public init(dbPath: String, mediaRoot: String) throws {
        handle = archival_db_open(dbPath, mediaRoot)
        guard handle != nil else {
            let msg = archival_last_error().map { String(cString: $0) } ?? "unknown"
            throw ArchivalDBError.openFailed(msg)
        }
    }

    deinit {
        if let h = handle {
            archival_db_close(h)
        }
    }

    // MARK: - Items

    public func createItem(category: ArchivalCategory) throws -> String {
        var outId: UnsafeMutablePointer<CChar>? = nil
        let rc = archival_item_create(handle, category.rawValue, &outId)
        guard rc == Ok, let ptr = outId else {
            throw ArchivalDBError.fromResult(rc, handle: handle)
        }
        defer { archival_free_string(ptr) }
        return String(cString: ptr)
    }

    public func deleteItem(id: String) throws {
        let rc = archival_item_delete(handle, id)
        if rc != Ok { throw ArchivalDBError.fromResult(rc, handle: handle) }
    }

    public func setFields(itemId: String, fields: [String: String?]) throws {
        let json = encodeFieldsJSON(fields)
        let rc = archival_item_set_fields(handle, itemId, json)
        if rc != Ok { throw ArchivalDBError.fromResult(rc, handle: handle) }
    }

    public func getFields(itemId: String) throws -> [String: String?] {
        var outJson: UnsafeMutablePointer<CChar>? = nil
        let rc = archival_item_get_fields(handle, itemId, &outJson)
        guard rc == Ok, let ptr = outJson else {
            throw ArchivalDBError.fromResult(rc, handle: handle)
        }
        defer { archival_free_string(ptr) }
        let json = String(cString: ptr)
        return try decodeFieldsJSON(json)
    }

    public func listItems(category: ArchivalCategory? = nil) throws -> [ArchivalItemSummary] {
        var outJson: UnsafeMutablePointer<CChar>? = nil
        let rc = archival_items_list(handle, category?.rawValue, &outJson)
        guard rc == Ok, let ptr = outJson else {
            throw ArchivalDBError.fromResult(rc, handle: handle)
        }
        defer { archival_free_string(ptr) }
        let json = String(cString: ptr)
        let data = Data(json.utf8)
        return try JSONDecoder().decode([ArchivalItemSummary].self, from: data)
    }

    // MARK: - Media

    public func addMedia(itemId: String, relativePath: String, mimeType: String, isPrimary: Bool) throws {
        let rc = archival_media_add(handle, itemId, relativePath, mimeType, isPrimary ? 1 : 0)
        if rc != Ok { throw ArchivalDBError.fromResult(rc, handle: handle) }
    }

    public func listMedia(itemId: String) throws -> [ArchivalMediaFile] {
        var outJson: UnsafeMutablePointer<CChar>? = nil
        let rc = archival_media_list(handle, itemId, &outJson)
        guard rc == Ok, let ptr = outJson else {
            throw ArchivalDBError.fromResult(rc, handle: handle)
        }
        defer { archival_free_string(ptr) }
        let data = Data(String(cString: ptr).utf8)
        return try JSONDecoder().decode([ArchivalMediaFile].self, from: data)
    }

    // MARK: - Tags

    public func addTag(itemId: String, tag: String) throws {
        let rc = archival_tag_add(handle, itemId, tag)
        if rc != Ok { throw ArchivalDBError.fromResult(rc, handle: handle) }
    }

    public func removeTag(itemId: String, tag: String) throws {
        let rc = archival_tag_remove(handle, itemId, tag)
        if rc != Ok { throw ArchivalDBError.fromResult(rc, handle: handle) }
    }

    public func tags(itemId: String) throws -> [String] {
        var outJson: UnsafeMutablePointer<CChar>? = nil
        let rc = archival_tags_for_item(handle, itemId, &outJson)
        guard rc == Ok, let ptr = outJson else {
            throw ArchivalDBError.fromResult(rc, handle: handle)
        }
        defer { archival_free_string(ptr) }
        let data = Data(String(cString: ptr).utf8)
        return try JSONDecoder().decode([String].self, from: data)
    }

    // MARK: - AI

    public func classifyImage(imagePath: String, apiKey: String) throws -> String {
        var outJson: UnsafeMutablePointer<CChar>? = nil
        let rc = archival_ai_classify(handle, imagePath, apiKey, &outJson)
        guard rc == Ok, let ptr = outJson else {
            throw ArchivalDBError.fromResult(rc, handle: handle)
        }
        defer { archival_free_string(ptr) }
        // Parse {"category":"Book"} → "Book"
        let json = String(cString: ptr)
        let data = Data(json.utf8)
        let obj = try JSONDecoder().decode([String: String].self, from: data)
        return obj["category"] ?? "Object"
    }

    public func fillFields(itemId: String, imagePath: String, apiKey: String) throws -> [String: String?] {
        var outJson: UnsafeMutablePointer<CChar>? = nil
        let rc = archival_ai_fill_fields(handle, itemId, imagePath, apiKey, &outJson)
        guard rc == Ok, let ptr = outJson else {
            throw ArchivalDBError.fromResult(rc, handle: handle)
        }
        defer { archival_free_string(ptr) }
        return try decodeFieldsJSON(String(cString: ptr))
    }

    // MARK: - JSON Helpers

    private func encodeFieldsJSON(_ fields: [String: String?]) -> String {
        var parts: [String] = []
        for (k, v) in fields {
            let escaped = k.replacingOccurrences(of: "\"", with: "\\\"")
            if let value = v {
                let vEscaped = value.replacingOccurrences(of: "\"", with: "\\\"")
                parts.append("\"\(escaped)\":\"\(vEscaped)\"")
            } else {
                parts.append("\"\(escaped)\":null")
            }
        }
        return "{\(parts.joined(separator: ","))}"
    }

    private func decodeFieldsJSON(_ json: String) throws -> [String: String?] {
        let data = Data(json.utf8)
        let raw = try JSONSerialization.jsonObject(with: data) as? [String: Any?] ?? [:]
        return raw.mapValues { v in v as? String }
    }
}

// MARK: - App Container Path Helper

public extension ArchivalDB {
    static func defaultPaths() -> (dbPath: String, mediaRoot: String) {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let dbPath    = docs.appendingPathComponent("archival.db").path
        let mediaRoot = docs.appendingPathComponent("media").path
        try? FileManager.default.createDirectory(atPath: mediaRoot, withIntermediateDirectories: true)
        return (dbPath, mediaRoot)
    }
}

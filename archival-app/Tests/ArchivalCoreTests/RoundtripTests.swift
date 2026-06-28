import XCTest
import ArchivalCore

@MainActor
final class RoundtripTests: XCTestCase {
    var tmpDir: URL!
    var db: ArchivalDB!

    override func setUp() async throws {
        tmpDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tmpDir, withIntermediateDirectories: true)
        let dbPath    = tmpDir.appendingPathComponent("test.db").path
        let mediaRoot = tmpDir.appendingPathComponent("media").path
        db = try ArchivalDB(dbPath: dbPath, mediaRoot: mediaRoot)
    }

    override func tearDown() async throws {
        db = nil
        try? FileManager.default.removeItem(at: tmpDir)
    }

    func testOpenClose() {
        XCTAssertNotNil(db)
    }

    func testCreateAndListBook() throws {
        let id = try db.createItem(category: .book)
        XCTAssertFalse(id.isEmpty)

        let items = try db.listItems()
        XCTAssertTrue(items.contains { $0.id == id })
    }

    func testSetAndGetFields() throws {
        let id = try db.createItem(category: .book)
        try db.setFields(itemId: id, fields: ["title": "Moby Dick", "author": "Herman Melville"])

        let fields = try db.getFields(itemId: id)
        XCTAssertEqual(fields["title"] as? String, "Moby Dick")
        XCTAssertEqual(fields["author"] as? String, "Herman Melville")
    }

    func testTagsRoundtrip() throws {
        let id = try db.createItem(category: .book)
        try db.addTag(itemId: id, tag: "childhood")
        try db.addTag(itemId: id, tag: "favorite")

        let tags = try db.tags(itemId: id)
        XCTAssertTrue(tags.contains("childhood"))
        XCTAssertTrue(tags.contains("favorite"))

        try db.removeTag(itemId: id, tag: "childhood")
        let tagsAfter = try db.tags(itemId: id)
        XCTAssertFalse(tagsAfter.contains("childhood"))
    }

    func testDeleteItem() throws {
        let id = try db.createItem(category: .music)
        try db.deleteItem(id: id)
        let items = try db.listItems()
        XCTAssertFalse(items.contains { $0.id == id })
    }

    func testAllCategoriesCreate() throws {
        for cat in ArchivalCategory.allCases {
            let id = try db.createItem(category: cat)
            XCTAssertFalse(id.isEmpty, "\(cat.rawValue) create failed")
        }
        let items = try db.listItems()
        XCTAssertEqual(items.count, ArchivalCategory.allCases.count)
    }

    func testAIStubs() throws {
        let id = try db.createItem(category: .book)

        let category = try db.classifyImage(imagePath: "/tmp/fake.jpg", apiKey: "stub")
        XCTAssertEqual(category, "Book")

        let fields = try db.fillFields(itemId: id, imagePath: "/tmp/fake.jpg", apiKey: "stub")
        XCTAssertNotNil(fields["title"])
    }
}

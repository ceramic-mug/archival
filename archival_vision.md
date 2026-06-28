# Archival

Cataloguing for the meaningful and mundane of everyday life. Cutting down on life's clutter while retaining and stimulating memories. This is Archival.

"Archival" is a system for ingesting, cataloguing, searching, and reliving memories of personal objects. It exists to make it easy to give away or dispose of posessions by making it easy to catalogue the memories tied to these posessions and return to them in the digital space.

Archival has four core functions:

- Ingestion and Indexing (AI-agent based)
- Database Maintenance
- Organized Display
- Export and Backup

The tech stack:

- Strage engine: Raw SQLite (C API)
- Core logic: Rust
- UI: SwiftUI
- AI: Google Gemini

Key principles:

- Application longevity. This is meant to be a lifelong archive. Hence the choice for storage and core logic that won't become obsolete, at least not within a few decades (hopefully)
- Ease of ingestion. It should be simple, quick, and straightforward to add well-indexed items to the archive. This will enable the application to be used regularly.

## Ingestion and Indexing

Easy photo-based indexing of objects. 

User takes a photo with their phone camera, or selects a photo from their computer. A photo-identification agent determines the nature of the object and performs rough categorization.

The application has a set of starter categories, which are themselves extensible, and new categories can be created by the user.

Starter categories include:

    - Book
    - Music
    - Movie
    - Game
    - Personal message (letter, card, etc.)
    - Award
    - Art
    - Photograph
    - Trinket
    - Jewelry
    - Clothing
    - Object

Where "Object" is a miscellaneous category and captures all literal objects not obviously contained in one of the other categories.

Each of these categories will be associated with a set of fields, like a class in object-oriented programming. For example, the class Book will have, by default, the following fields:

    - Title
    - Author
    - ISBN
    - Dewey decimal
    - Publication date
    - City of original publication
    - Country of original publication
    - Language of original publication
    - 2 sentence summary

A specialist agent will be assigned to each category -- in the case of category "Book," we will have agent "Librarian" -- with the necessary 

Each category will have a specialist agent that is prompted to auto-fill some of the fields, and some agents will have access to web search to fill the fields. For example, the photo-identification agent will determine the category and return a structured json with simply {category}, e.g., {book}. This will prompt the specialist agent with the photo, e.g., librarian, who will have access to the photo and necessary tools to search the web, to identify information for various fields, e.g., {title: {int}, author: {int}, isbn: {int}} Some fields will be blank by default for user entry. Each item of each category will contain the generating photo alongside the fields and basic metadata, such as creation timestamp.

## Maintenance of Highly Structured Database

A pure SQLite database, using the C API, will form the storage layer. Blobs (photos, other media) should not be stored here -- instead, pointers or references to flat files will form the bridge to media.

The application should prioritize maintenance, backwards compatibility, and ease of customization by the user, as well as natural-language based customization using a "databaser" agent with user validation/approval.

All standard CRUD operations should be built from the beginning. The database schema should be viewable by the user at any time for validation. The SQLite should be kept as raw and portable as possible for longevity's sake.

Tables, mapping, and indexing should conform to rigorous standards for high efficiency.

Beyond the core categories, the user should be able to sort and filter by contained text, applied tags, or custom sets or lists functioning as groups of items across categories (for example, books and movies from a user's "High School").

## Organized Display

We can worry about design details after core database and logic is established. The key priorities of the display are to:

- Make it easy for the user to browse the archive
- Easy and beautiful sorting

For a first iteration, a functional wrapper that exposes all of the lower API will be sufficient.

## Export and Backup

The user should have robust export options, allowing for the generation of beautiful, latex/pdf style reports of categories or custom sets, to be able to share with friends and family.
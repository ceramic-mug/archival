pub struct FieldDef {
    pub name: &'static str,
    pub sql_column: &'static str,
    pub ai_fillable: bool,
    pub user_entry: bool,
}

pub struct Category {
    pub name: &'static str,
    pub table: &'static str,
    pub specialist: &'static str,
    pub fields: &'static [FieldDef],
}

macro_rules! field {
    ($name:expr, $col:expr, ai) => {
        FieldDef { name: $name, sql_column: $col, ai_fillable: true, user_entry: false }
    };
    ($name:expr, $col:expr, user) => {
        FieldDef { name: $name, sql_column: $col, ai_fillable: false, user_entry: true }
    };
    ($name:expr, $col:expr, both) => {
        FieldDef { name: $name, sql_column: $col, ai_fillable: true, user_entry: true }
    };
}

pub static BOOK_FIELDS: &[FieldDef] = &[
    field!("Title",                    "title",          ai),
    field!("Author",                   "author",         ai),
    field!("ISBN",                     "isbn",           ai),
    field!("Dewey Decimal",            "dewey_decimal",  ai),
    field!("Publication Date",         "pub_date",       ai),
    field!("City of Publication",      "pub_city",       ai),
    field!("Country of Publication",   "pub_country",    ai),
    field!("Language",                 "pub_language",   ai),
    field!("Summary",                  "summary",        ai),
];

pub static MUSIC_FIELDS: &[FieldDef] = &[
    field!("Title",   "title",   ai),
    field!("Artist",  "artist",  ai),
    field!("Album",   "album",   ai),
    field!("Year",    "year",    ai),
    field!("Label",   "label",   ai),
    field!("Genre",   "genre",   ai),
    field!("Format",  "format",  both),
];

pub static MOVIE_FIELDS: &[FieldDef] = &[
    field!("Title",     "title",     ai),
    field!("Director",  "director",  ai),
    field!("Year",      "year",      ai),
    field!("Studio",    "studio",    ai),
    field!("Format",    "format",    both),
    field!("Language",  "language",  ai),
    field!("Synopsis",  "synopsis",  ai),
];

pub static GAME_FIELDS: &[FieldDef] = &[
    field!("Title",      "title",      ai),
    field!("Developer",  "developer",  ai),
    field!("Publisher",  "publisher",  ai),
    field!("Year",       "year",       ai),
    field!("Platform",   "platform",   both),
    field!("Genre",      "genre",      ai),
];

pub static PERSONAL_MESSAGE_FIELDS: &[FieldDef] = &[
    field!("Sender",    "sender",     user),
    field!("Recipient", "recipient",  user),
    field!("Date Sent", "date_sent",  user),
    field!("Medium",    "medium",     both),
    field!("Occasion",  "occasion",   user),
];

pub static AWARD_FIELDS: &[FieldDef] = &[
    field!("Title",         "title",          ai),
    field!("Issuing Org",   "issuing_org",    ai),
    field!("Date Received", "date_received",  user),
    field!("Category",      "category",       ai),
    field!("Description",   "description",    both),
];

pub static ART_FIELDS: &[FieldDef] = &[
    field!("Title",      "title",      both),
    field!("Artist",     "artist",     both),
    field!("Medium",     "medium",     both),
    field!("Year",       "year",       both),
    field!("Dimensions", "dimensions", user),
    field!("Provenance", "provenance", user),
];

pub static PHOTOGRAPH_FIELDS: &[FieldDef] = &[
    field!("Subject",      "subject",       both),
    field!("Date Taken",   "date_taken",    user),
    field!("Location",     "location",      both),
    field!("Photographer", "photographer",  user),
    field!("Format",       "format",        both),
];

pub static TRINKET_FIELDS: &[FieldDef] = &[
    field!("Name",             "name",              both),
    field!("Origin",           "origin",            both),
    field!("Material",         "material",          both),
    field!("Approximate Date", "approximate_date",  user),
];

pub static JEWELRY_FIELDS: &[FieldDef] = &[
    field!("Name",             "name",              both),
    field!("Material",         "material",          both),
    field!("Gemstones",        "gemstones",         both),
    field!("Maker",            "maker",             both),
    field!("Approximate Date", "approximate_date",  user),
];

pub static CLOTHING_FIELDS: &[FieldDef] = &[
    field!("Name",             "name",              both),
    field!("Brand",            "brand",             both),
    field!("Size",             "size",              user),
    field!("Material",         "material",          both),
    field!("Color",            "color",             both),
    field!("Approximate Date", "approximate_date",  user),
];

pub static OBJECT_FIELDS: &[FieldDef] = &[
    field!("Name",             "name",              both),
    field!("Material",         "material",          both),
    field!("Manufacturer",     "manufacturer",      both),
    field!("Approximate Date", "approximate_date",  user),
    field!("Description",      "description",       both),
];

pub static CATEGORIES: &[Category] = &[
    Category { name: "Book",            table: "book_fields",             specialist: "Librarian",    fields: BOOK_FIELDS            },
    Category { name: "Music",           table: "music_fields",            specialist: "DJ",           fields: MUSIC_FIELDS           },
    Category { name: "Movie",           table: "movie_fields",            specialist: "Critic",       fields: MOVIE_FIELDS           },
    Category { name: "Game",            table: "game_fields",             specialist: "Archivist",    fields: GAME_FIELDS            },
    Category { name: "PersonalMessage", table: "personal_message_fields", specialist: "Correspondent",fields: PERSONAL_MESSAGE_FIELDS },
    Category { name: "Award",           table: "award_fields",            specialist: "Curator",      fields: AWARD_FIELDS           },
    Category { name: "Art",             table: "art_fields",              specialist: "Appraiser",    fields: ART_FIELDS             },
    Category { name: "Photograph",      table: "photograph_fields",       specialist: "Archivist",    fields: PHOTOGRAPH_FIELDS      },
    Category { name: "Trinket",         table: "trinket_fields",          specialist: "Curator",      fields: TRINKET_FIELDS         },
    Category { name: "Jewelry",         table: "jewelry_fields",          specialist: "Appraiser",    fields: JEWELRY_FIELDS         },
    Category { name: "Clothing",        table: "clothing_fields",         specialist: "Curator",      fields: CLOTHING_FIELDS        },
    Category { name: "Object",          table: "object_fields",           specialist: "Archivist",    fields: OBJECT_FIELDS          },
];

pub fn find_category(name: &str) -> Option<&'static Category> {
    CATEGORIES.iter().find(|c| c.name == name)
}

pub fn valid_category_names() -> Vec<&'static str> {
    CATEGORIES.iter().map(|c| c.name).collect()
}

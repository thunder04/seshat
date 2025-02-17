use const_format::formatcp;

pub trait SqlQueries: Send + Sync + 'static {
    fn retrieve_books(&self) -> &'static str;
    fn retrieve_book_authors(&self) -> &'static str;
    fn retrieve_book_languages(&self) -> &'static str;
    fn retrieve_book_tags(&self) -> &'static str;
    fn retrieve_book_data(&self) -> &'static str;
}

macro_rules! impl_sql_queries {
    ($($struct_name: ident: [order_by: $order_by: literal]),+ $(,)?) => {$(
        pub struct $struct_name;

        impl SqlQueries for $struct_name {
            fn retrieve_books(&self) -> &'static str {
                const {
                    formatcp!(
                        r#"SELECT
                           	b.id AS id,
                           	b.uuid AS uuid,
                           	b.title AS title,
                           	b.timestamp AS added_at,
                           	b.pubdate AS published_at,
                           	b.has_cover AS has_cover,
                           	b.last_modified AS last_modified_at,
                           	b.path AS path,
                           	c.text AS comment
                        FROM books as b
                  		LEFT JOIN comments as c ON c.book = b.id
                       	ORDER BY {order_by}
                       	LIMIT ?1 OFFSET ?2"#,
                        order_by = $order_by
                    )
                }
            }

            fn retrieve_book_authors(&self) -> &'static str {
                const {
                    formatcp!(
                        r#"SELECT
                            a.name AS author_name,
                            link.book AS book_id
                        FROM books_authors_link as link
                       	INNER JOIN (
                      		SELECT id AS b_id FROM books
                      		ORDER BY {order_by}
                      		LIMIT ?1 OFFSET ?2
                       	) ON book_id = b_id
                       	INNER JOIN authors AS a ON link.author = a.id;"#,
                        order_by = $order_by
                    )
                }
            }

            fn retrieve_book_languages(&self) -> &'static str {
                const {
                    formatcp!(
                        r#"SELECT
                            l.lang_code AS lang_code,
                            link.book AS book_id
                        FROM books_languages_link as link
                       	INNER JOIN (
                      		SELECT id AS b_id FROM books
                      		ORDER BY {order_by}
                      		LIMIT ?1 OFFSET ?2
                       	) ON book_id = b_id
                       	INNER JOIN languages AS l ON link.lang_code = l.id;"#,
                        order_by = $order_by
                    )
                }
            }

            fn retrieve_book_tags(&self) -> &'static str {
                const {
                    formatcp!(
                        r#"SELECT
                            link.book AS book_id,
                            t.name AS tag_name
                        FROM books_tags_link as link
                       	INNER JOIN (
                      		SELECT id AS b_id FROM books
                      		ORDER BY {order_by}
                      		LIMIT ?1 OFFSET ?2
                       	) ON book_id = b_id
                       	INNER JOIN tags AS t ON link.tag = t.id;"#,
                        order_by = $order_by
                    )
                }
            }

            fn retrieve_book_data(&self) -> &'static str {
                const {
                    formatcp!(
                        r#"SELECT
                            d.uncompressed_size AS file_size,
                            d.name AS file_name,
                            d.format AS format,
                            d.book AS book_id
                        FROM data AS d
                       	WHERE book_id IN (
                            SELECT
                                id AS b_id
                            FROM books
                            ORDER BY {order_by}
                            LIMIT ?1 OFFSET ?2);"#,
                        order_by = $order_by
                    )
                }
            }
        }
    )+};
}

impl_sql_queries! {
    OrderedByDateAdded: [order_by: "timestamp DESC"],
    OrderedByAuthor: [order_by: "author_sort ASC"],
    OrderedByTitle: [order_by: "sort ASC"],
}

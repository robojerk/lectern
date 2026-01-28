use crate::ui::{Lectern, Message};
use iced::Command;

pub fn handle_metadata(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::TitleChanged(title) => {
            app.metadata.editing_title = title.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.title = title;
            }
            Some(Command::none())
        }
        Message::AuthorChanged(author) => {
            app.metadata.editing_author = author.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.author = author;
            }
            Some(Command::none())
        }
        Message::SeriesChanged(series) => {
            app.metadata.editing_series = series.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.series = Some(series);
            }
            Some(Command::none())
        }
        Message::NarratorChanged(narrator) => {
            app.metadata.editing_narrator = narrator.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.narrator = Some(narrator);
            }
            Some(Command::none())
        }
        Message::DescriptionAction(action) => {
            app.metadata.editing_description_content.perform(action);
            let text = app.metadata.editing_description_content.text();
            app.metadata.editing_description = text.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.description = Some(text);
            }
            Some(Command::none())
        }
        Message::SubtitleChanged(subtitle) => {
            app.metadata.editing_subtitle = subtitle.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.subtitle = Some(subtitle);
            }
            Some(Command::none())
        }
        Message::SeriesNumberChanged(num) => {
            app.metadata.editing_series_number = num.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.series_number = Some(num);
            }
            Some(Command::none())
        }
        Message::IsbnChanged(isbn) => {
            app.metadata.editing_isbn = isbn.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.isbn = Some(isbn);
            }
            Some(Command::none())
        }
        Message::AsinChanged(asin) => {
            app.metadata.editing_asin = asin.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.asin = if asin.trim().is_empty() {
                    None
                } else {
                    Some(asin)
                };
            }
            Some(Command::none())
        }
        Message::PublisherChanged(pub_name) => {
            app.metadata.editing_publisher = pub_name.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.publisher = Some(pub_name);
            }
            Some(Command::none())
        }
        Message::PublishYearChanged(year) => {
            app.metadata.editing_publish_year = year.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.publish_year = Some(year);
            }
            Some(Command::none())
        }
        Message::GenreChanged(genre) => {
            app.metadata.editing_genre = genre.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.genre = Some(genre);
            }
            Some(Command::none())
        }
        Message::TagsChanged(tags) => {
            app.metadata.editing_tags = tags.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.tags = Some(tags);
            }
            Some(Command::none())
        }
        Message::LanguageChanged(lang) => {
            app.metadata.editing_language = lang.clone();
            if let Some(ref mut book) = app.metadata.selected_book {
                book.language = Some(lang);
            }
            Some(Command::none())
        }
        Message::ExplicitToggled(value) => {
            app.metadata.editing_explicit = value;
            if let Some(ref mut book) = app.metadata.selected_book {
                book.explicit = Some(value);
            }
            Some(Command::none())
        }
        Message::AbridgedToggled(value) => {
            app.metadata.editing_abridged = value;
            if let Some(ref mut book) = app.metadata.selected_book {
                book.abridged = Some(value);
            }
            Some(Command::none())
        }
        Message::MetadataProviderChanged(provider) => {
            println!("[DEBUG] MetadataProviderChanged to: {:?}", provider);
            app.metadata.metadata_provider = provider;
            Some(Command::none())
        }
        _ => None,
    }
}

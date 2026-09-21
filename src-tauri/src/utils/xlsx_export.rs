use crate::utils::cow_filter::CowFilter;

pub fn filter_to_messages(filter: &CowFilter) -> Vec<String> {
    let mut messages = Vec::new();

    let has_any_filter = filter.last_4_digits_eartag.is_some() || filter.ear_tag_contains.is_some() ||
                         filter.breed.is_some() || 
                         filter.sex.is_some() ||
                         filter.born_in_year.is_some() || 
                         filter.born_on.is_some() ||
                         filter.minimum_age_months.is_some() || 
                         filter.maximum_age_months.is_some() ||
                         filter.entered_on.is_some() || 
                         filter.exited_on.is_some() || 
                         filter.category.is_some() ||
                         filter.births_less_than.is_some() ||
                         filter.births_more_than.is_some() ||
                         filter.inseminations_less_than.is_some() ||
                         filter.inseminations_more_than.is_some();

    // 1. Reference Point (Time Travel)
    if let Some(d) = filter.date {
        messages.push(format!("Situație la data de: {}", d.format("%d.%m.%Y")));
    }
    messages.push("Filtre aplicate:".to_string());
    if filter.show_only_entered {
        messages.push("- Stare: în fermă la data de referință (bovinele ieșite în acea zi nu sunt incluse).".to_string());
    } else if filter.show_only_exited {
        messages.push("- Stare: bovine ieșite până la data de referință, inclusiv.".to_string());
    } else {
        messages.push("- Stare: toate bovinele din istoricul fermei.".to_string());
    }

    if !has_any_filter {
        messages.push("Nu sunt aplicate alte filtre.".to_string());
    } else {
        
        // --- Identity & Physical Traits ---
        if let Some(text) = &filter.ear_tag_contains {
            messages.push(format!("- Crotalia să conțină: {}", text));
        }
        if let Some(tag) = &filter.last_4_digits_eartag {
            messages.push(format!("- Să aibă o crotalie ce se termină în {}", tag));
        }
        if let Some(b) = &filter.breed {
            messages.push(format!("- Să aibă rasa: {}", b));
        }
        if let Some(s) = &filter.sex {
            let s_str = if s.to_string() == "Male" || s.to_string() == "M" { "Mascul" } else { "Femelă" };
            messages.push(format!("- Să aibă sexul: {}", s_str));
        }
        if let Some(c) = &filter.category {
            messages.push(format!("- Să aibă categoria: {}", c));
        }

        // --- Age & Birth Date ---
        if let Some(y) = filter.born_in_year {
            messages.push(format!("- Să fie născută în anul: {}", y));
        }
        if let Some(d) = filter.born_on {
            messages.push(format!("- Să fie născută pe data de: {}", d.format("%d.%m.%Y")));
        }
        if let Some(max) = filter.maximum_age_months {
            messages.push(format!("- Vârsta sub {} luni împlinite (strict mai mică)", max));
        }
        if let Some(min) = filter.minimum_age_months {
            messages.push(format!("- Vârsta peste {} luni împlinite (mai mare sau egală)", min));
        }

        // --- Lifecycle Events ---
        if let Some(e) = filter.entered_on {
            messages.push(format!("- Să fi intrat pe data de: {}", e.format("%d.%m.%Y")));
        }
        if let Some(ex) = filter.exited_on {
            messages.push(format!("- Să fi ieșit pe data de: {}", ex.format("%d.%m.%Y")));
        }

        // --- Production/Relational Filters ---
        if let Some(lt) = filter.births_less_than {
            messages.push(format!("- Număr de fătări mai mic de: {}", lt));
        }
        if let Some(mt) = filter.births_more_than {
            messages.push(format!("- Număr de fătări mai mare de: {}", mt));
        }
        if let Some(lt) = filter.inseminations_less_than {
            messages.push(format!("- Număr de însămânțări mai mic de: {}", lt));
        }
        if let Some(mt) = filter.inseminations_more_than {
            messages.push(format!("- Număr de însămânțări mai mare de: {}", mt));
        }
    }

    messages
}

use rust_xlsxwriter::{Workbook, Format, XlsxError, 
    FormatBorder, FormatAlign
};
use crate::model::cow::Cow;

pub fn write_to_xlsx(path: &str, messages: Vec<String>, cows: Vec<Cow>) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name("Bovine")?;

    // --- Styles ---
    let border_style = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);

    let header_style = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_bold();

    let message_style = Format::new()
        .set_bold()
        .set_font_size(14);

    // --- Page Setup ---
    worksheet.set_paper_size(4);
    worksheet.set_margins(0.2, 0.2, 0.75, 0.75, 0.3, 0.3);
    worksheet.set_print_fit_to_pages(1, 0);

    let mut row_num: u32 = 0;
    let column_count = 10; // Updated from 8 to 10

    // --- Write Messages ---
    for message in messages {
        worksheet.merge_range(row_num, 0, row_num, column_count - 1, &message, &message_style)?;
        row_num += 1;
    }
    
    row_num += 2;

    // --- Header Row ---
    let headers = [
        "Nr.", "Crotalie", "Rasă", "Sex", "Dată Naștere", 
        "Dată Intrare", "Dată Ieșire", "Categorie", "Fătări", "Însămânțări"
    ];

    for (i, text) in headers.iter().enumerate() {
        worksheet.write_string(row_num, i as u16, *text)?;
        worksheet.set_cell_format(row_num, i as u16, &header_style)?;
    }
    
    row_num += 1;

    // --- Data Rows ---
    for (idx, cow) in cows.iter().enumerate() {
        let row = row_num;
        let current_idx = (idx + 1) as f64;
        
        worksheet.write_number(row, 0, current_idx)?;
        worksheet.write_string(row, 1, &cow.ear_tag)?;
        worksheet.write_string(row, 2, match cow.breed {
            crate::model::cow::Breed::Metis => "Metis",
            crate::model::cow::Breed::BaltataRomaneasca => "Bălțată românească",
            crate::model::cow::Breed::AmbardeenAngus => "Aberdeen Angus",
        })?;
        worksheet.write_string(row, 3, match cow.sex {
            crate::model::cow::Sex::Male => "Mascul",
            crate::model::cow::Sex::Female => "Femelă",
        })?;
        worksheet.write_string(row, 4, &cow.birth_date.format("%d.%m.%Y").to_string())?;
        worksheet.write_string(row, 5, &cow.entry_date.format("%d.%m.%Y").to_string())?;
        
        let exit_str = cow.exit_date.map(|d| d.format("%d.%m.%Y").to_string()).unwrap_or_default();
        worksheet.write_string(row, 6, &exit_str)?;
        
        worksheet.write_string(row, 7, &cow.category.to_string())?;
        
        // New Count Columns
        worksheet.write_number(row, 8, cow.birth_count as f64)?;
        worksheet.write_number(row, 9, cow.insemination_count as f64)?;
        
        // Apply borders to the entire row (0 to 9)
        for col in 0..column_count {
            worksheet.set_cell_format(row, col as u16, &border_style)?;
        }
        
        row_num += 1;
    }

    worksheet.autofit();
    // Keep the numbering column compact regardless of the report headings.
    worksheet.set_column_width(0, 6)?;
    workbook.save(path)?;
    Ok(())
}

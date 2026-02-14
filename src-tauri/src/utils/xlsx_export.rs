use crate::utils::cow_filter::CowFilter;

pub fn filter_to_messages(filter: &CowFilter) -> Vec<String> {
    let mut messages = Vec::new();

    let has_any_filter = filter.last_4_digits_eartag.is_some() || 
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

    if !has_any_filter {
        if(filter.show_only_entered){
            messages.push("Bovinele din fermă sunt următoarele:".to_string());
        }
        else {
            messages.push("Bovinele din istoricul fermei sunt următoarele:".to_string());
        }
    } else {
        if(filter.show_only_entered){
            messages.push("Bovinele din fermă care îndeplinesc următoarele criteriisunt următoarele:".to_string());
        }
        else {
            messages.push("Bovinele din istoricul fermei care satisfac următoarele criterii sunt următoarele:".to_string());
        }
        
        // --- Identity & Physical Traits ---
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
        if let Some(min) = filter.minimum_age_months {
            messages.push(format!("- Vârsta minimă (luni): {}", min));
        }
        if let Some(max) = filter.maximum_age_months {
            messages.push(format!("- Vârsta maximă (luni): {}", max));
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
    let worksheet = workbook.add_worksheet().set_name("Vaci")?;

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

    // --- Print Setup ---
    worksheet.set_paper_size(4);
    worksheet.set_portrait(); // Portrait mode
    worksheet.set_margins(0.2, 0.2, 0.75, 0.75, 0.3, 0.3); // Left, Right, Top, Bottom
    worksheet.set_print_fit_to_pages(1, 0); // Fit to 1 page wide, auto height

    let mut row_num: u32 = 0;
    let column_count = 8;

    // --- Write Messages ---
    for message in messages {
        worksheet.merge_range(row_num, 0, row_num, column_count - 1, &message, &message_style)?;
        row_num += 1;
    }
    
    row_num += 4; // Spacing before the table

    // --- Header Row ---
    let headers = ["Nr.", "Crotalie", "Rasă", "Sex", "Dată Naștere", "Dată Intrare", "Dată Ieșire", "Categorie"];
    for (i, text) in headers.iter().enumerate() {
        worksheet.write_string_with_format(row_num, i as u16, *text, &header_style)?;
    }
    
    let table_start_row = row_num;
    row_num += 1;

    // --- Data Rows ---
    for (idx, cow) in cows.iter().enumerate() {
        let current_idx = (idx + 1) as u32;
        
        worksheet.write_number_with_format(row_num, 0, current_idx as f64, &border_style)?;
        worksheet.write_string_with_format(row_num, 1, &cow.ear_tag, &border_style)?;
        worksheet.write_string_with_format(row_num, 2, &cow.breed.to_string(), &border_style)?;
        worksheet.write_string_with_format(row_num, 3, &cow.sex.to_string(), &border_style)?;
        worksheet.write_string_with_format(row_num, 4, &cow.birth_date.format("%d.%m.%Y").to_string(), &border_style)?;
        worksheet.write_string_with_format(row_num, 5, &cow.entry_date.format("%d.%m.%Y").to_string(), &border_style)?;
        
        let exit_str = cow.exit_date.map(|d| d.format("%d.%m.%Y").to_string()).unwrap_or_default();
        worksheet.write_string_with_format(row_num, 6, &exit_str, &border_style)?;
        
        worksheet.write_string_with_format(row_num, 7, &cow.category.to_string(), &border_style)?;
        
        row_num += 1;
    }

    // --- Auto-size Columns ---
    worksheet.autofit();

    workbook.save(path)?;
    Ok(())
}
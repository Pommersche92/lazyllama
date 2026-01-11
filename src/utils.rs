/*
 *  _                      _      _
 * | |    __ _  ______  __| |    | | __ _ _ __ ___   __ _
 * | |   / _` ||_  /\ \/ /| |    | |/ _` | '_ ` _ \ / _` |
 * | |__| (_| | / /  \  / | |___ | | (_| | | | | | | (_| |
 * |_____\__,_|/___| /_/  |_____||_|\__,_|_| |_| |_|\__,_|
 *
 * Copyright (C) 2026 Raimo Geisel
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

//! Hilfsfunktionen für Dateioperationen und Systemzugriffe.

use anyhow::Result;
use chrono::Local;
use std::fs;

/// Speichert den übergebenen Text in einer neuen Datei im lokalen Datenverzeichnis.
///
/// Die Datei wird unter `~/.local/share/lazyllama/chat_DATUM_UHRZEIT.txt` abgelegt.
/// Gibt `Ok(())` zurück, wenn die Datei erfolgreich geschrieben wurde oder der Verlauf leer war.
pub fn save_history_to_file(history: &str) -> Result<()> {
    if history.is_empty() { return Ok(()); }
    let mut log_dir = dirs::data_local_dir().ok_or_else(|| anyhow::anyhow!("Data dir not found"))?;
    log_dir.push("lazyllama");
    fs::create_dir_all(&log_dir)?;
    let filename = format!("chat_{}.txt", Local::now().format("%Y-%m-%d_%H-%M-%S"));
    log_dir.push(filename);
    fs::write(log_dir, history)?;
    Ok(())
}

// Database Operations for the ConPort MCP server
use rusqlite::{Connection, params};
use serde_json::Value;

use crate::Type::Progress::{
    Progress, ProgressResponse, LogProgressArgs, GetProgressArgs, UpdateProgressArgs,
};
use crate::Type::SystemPattern::{
    SystemPattern, SystemPatternResponse, LogSystemPatternArgs, GetSystemPatternsArgs,
};
use crate::Type::CustomData::{
    CustomData, CustomDataResponse, LogCustomDataArgs, GetCustomDataArgs, DeleteCustomDataArgs,
    SearchCustomDataValueArgs,
};
use crate::Type::History::{ContextHistory, HistoryResponse, GetItemHistoryArgs, item_type};
use crate::Type::ContextLink::{
    ContextLink, LinkResponse, LinkConportItemsArgs, GetLinkedItemsArgs,
};
use crate::Type::Decision::{
    Decision, DecisionResponse, LogDecisionArgs, GetDecisionsArgs, UpdateDecisionArgs,
    DeleteDecisionArgs, SearchDecisionsArgs,
};
use super::Connect::current_timestamp;

// ============================================================================
// Progress Entry Operations
// ============================================================================

/// Log a new progress entry
pub fn log_progress(
    conn: &Connection,
    args: &LogProgressArgs,
) -> Result<Progress, String> {
    let timestamp = current_timestamp();
    
    conn.execute(
        "INSERT INTO ProgressEntries (WorkspaceId, Timestamp, Status, Description, ParentId, CreatedAt, UpdatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            args.workspace_id,
            timestamp,
            args.status,
            args.description,
            args.parent_id,
            timestamp,
            timestamp,
        ],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    
    Ok(Progress {
        Id: id.to_string(),
        WorkspaceId: args.workspace_id.clone(),
        Timestamp: timestamp.clone(),
        Status: args.status.clone(),
        Description: args.description.clone(),
        ParentId: args.parent_id,
        CreatedAt: timestamp.clone(),
        UpdatedAt: timestamp,
    })
}

/// Get progress entries with filters
pub fn get_progress(
    conn: &Connection,
    workspace_id: &str,
    args: &GetProgressArgs,
) -> Result<Vec<Progress>, String> {
    let mut sql = String::from(
        "SELECT Id, WorkspaceId, Timestamp, Status, Description, ParentId, CreatedAt, UpdatedAt 
         FROM ProgressEntries WHERE WorkspaceId = ?1"
    );
    
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(workspace_id.to_string())];
    
    if let Some(ref status) = args.status_filter {
        sql.push_str(" AND Status = ?");
        params_vec.push(Box::new(status.clone()));
    }
    
    if let Some(parent_id) = args.parent_id_filter {
        sql.push_str(" AND ParentId = ?");
        params_vec.push(Box::new(parent_id));
    }
    
    sql.push_str(" ORDER BY Timestamp DESC");
    
    if let Some(limit) = args.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(Progress {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Status: row.get(3)?,
            Description: row.get(4)?,
            ParentId: row.get(5)?,
            CreatedAt: row.get(6)?,
            UpdatedAt: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(results)
}

/// Update a progress entry
pub fn update_progress(
    conn: &Connection,
    workspace_id: &str,
    args: &UpdateProgressArgs,
) -> Result<bool, String> {
    let mut updates = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(workspace_id.to_string())];
    
    if let Some(ref status) = args.status {
        updates.push("Status = ?");
        params_vec.push(Box::new(status.clone()));
    }
    
    if let Some(ref description) = args.description {
        updates.push("Description = ?");
        params_vec.push(Box::new(description.clone()));
    }
    
    if args.parent_id.is_some() {
        updates.push("ParentId = ?");
        params_vec.push(Box::new(args.parent_id));
    }
    
    if updates.is_empty() {
        return Ok(false);
    }
    
    // Always update UpdatedAt
    updates.push("UpdatedAt = ?");
    params_vec.push(Box::new(current_timestamp()));
    
    // Add progress_id to params
    params_vec.push(Box::new(args.progress_id));
    
    let sql = format!(
        "UPDATE ProgressEntries SET {} WHERE Id = ? AND WorkspaceId = ?",
        updates.join(", ")
    );
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let rows = conn.execute(&sql, params_refs.as_slice()).map_err(|e| e.to_string())?;
    
    Ok(rows > 0)
}

/// Delete a progress entry by ID
pub fn delete_progress(
    conn: &Connection,
    workspace_id: &str,
    progress_id: i64,
) -> Result<bool, String> {
    let rows = conn.execute(
        "DELETE FROM ProgressEntries WHERE Id = ? AND WorkspaceId = ?",
        params![progress_id, workspace_id],
    ).map_err(|e| e.to_string())?;
    
    Ok(rows > 0)
}

// ============================================================================
// System Pattern Operations
// ============================================================================

/// Log a new system pattern
pub fn log_system_pattern(
    conn: &Connection,
    args: &LogSystemPatternArgs,
) -> Result<SystemPattern, String> {
    let timestamp = current_timestamp();
    let tags_json = args.tags.as_ref()
        .map(|tags| serde_json::to_string(tags).unwrap_or_default())
        .unwrap_or_default();
    
    conn.execute(
        "INSERT INTO SystemPatterns (WorkspaceId, Timestamp, Name, Description, Tags, CreatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            args.workspace_id,
            timestamp,
            args.name,
            args.description,
            tags_json,
            timestamp,
        ],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    
    Ok(SystemPattern {
        Id: id.to_string(),
        WorkspaceId: args.workspace_id.clone(),
        Timestamp: timestamp.clone(),
        Name: args.name.clone(),
        Description: args.description.clone(),
        Tags: args.tags.clone(),
        CreatedAt: timestamp,
    })
}

/// Get system patterns with filters
pub fn get_system_patterns(
    conn: &Connection,
    workspace_id: &str,
    args: &GetSystemPatternsArgs,
) -> Result<Vec<SystemPattern>, String> {
    let mut sql = String::from(
        "SELECT Id, WorkspaceId, Timestamp, Name, Description, Tags, CreatedAt 
         FROM SystemPatterns WHERE WorkspaceId = ?1"
    );
    
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(workspace_id.to_string())];
    
    // Handle tag filters
    if let Some(ref tags) = args.tags_filter_include_all {
        for tag in tags {
            sql.push_str(" AND Tags LIKE ?");
            params_vec.push(Box::new(format!("%\"{}%", tag)));
        }
    } else if let Some(ref tags) = args.tags_filter_include_any {
        let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
        sql.push_str(&format!(" AND (Tags LIKE {} OR Tags IS NULL)", placeholders.join(" OR Tags LIKE ")));
        for tag in tags {
            params_vec.push(Box::new(format!("%\"{}%", tag)));
        }
    }
    
    sql.push_str(" ORDER BY Timestamp DESC");
    
    if let Some(limit) = args.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        let tags_str: Option<String> = row.get(5)?;
        let tags: Option<Vec<String>> = tags_str
            .and_then(|s| serde_json::from_str(&s).ok());
        
        Ok(SystemPattern {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Name: row.get(3)?,
            Description: row.get(4)?,
            Tags: tags,
            CreatedAt: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(results)
}

/// Delete a system pattern by ID
pub fn delete_system_pattern(
    conn: &Connection,
    workspace_id: &str,
    pattern_id: i64,
) -> Result<bool, String> {
    let rows = conn.execute(
        "DELETE FROM SystemPatterns WHERE Id = ? AND WorkspaceId = ?",
        params![pattern_id, workspace_id],
    ).map_err(|e| e.to_string())?;
    
    Ok(rows > 0)
}

// ============================================================================
// Custom Data Operations
// ============================================================================

/// Log custom data
pub fn log_custom_data(
    conn: &Connection,
    args: &LogCustomDataArgs,
) -> Result<CustomData, String> {
    let timestamp = current_timestamp();
    let value_json = serde_json::to_string(&args.value).unwrap_or_default();
    
    conn.execute(
        "INSERT INTO CustomData (WorkspaceId, Timestamp, Category, Key, Value, CreatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            args.workspace_id,
            timestamp,
            args.category,
            args.key,
            value_json,
            timestamp,
        ],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    
    Ok(CustomData {
        Id: id.to_string(),
        WorkspaceId: args.workspace_id.clone(),
        Timestamp: timestamp.clone(),
        Category: args.category.clone(),
        Key: args.key.clone(),
        Value: args.value.clone(),
        CreatedAt: timestamp,
    })
}

/// Get custom data with filters
pub fn get_custom_data(
    conn: &Connection,
    workspace_id: &str,
    args: &GetCustomDataArgs,
) -> Result<Vec<CustomData>, String> {
    let mut sql = String::from(
        "SELECT Id, WorkspaceId, Timestamp, Category, Key, Value, CreatedAt 
         FROM CustomData WHERE WorkspaceId = ?1"
    );
    
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(workspace_id.to_string())];
    
    if let Some(ref category) = args.category {
        sql.push_str(" AND Category = ?");
        params_vec.push(Box::new(category.clone()));
        
        if let Some(ref key) = args.key {
            sql.push_str(" AND Key = ?");
            params_vec.push(Box::new(key.clone()));
        }
    }
    
    sql.push_str(" ORDER BY Timestamp DESC");
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        let value_str: String = row.get(5)?;
        let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::Null);
        
        Ok(CustomData {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Category: row.get(3)?,
            Key: row.get(4)?,
            Value: value,
            CreatedAt: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(results)
}

/// Delete custom data
pub fn delete_custom_data(
    conn: &Connection,
    workspace_id: &str,
    category: &str,
    key: &str,
) -> Result<bool, String> {
    let rows = conn.execute(
        "DELETE FROM CustomData WHERE WorkspaceId = ? AND Category = ? AND Key = ?",
        params![workspace_id, category, key],
    ).map_err(|e| e.to_string())?;
    
    Ok(rows > 0)
}

/// Search custom data using FTS5
pub fn search_custom_data_fts(
    conn: &Connection,
    workspace_id: &str,
    query_term: &str,
    category_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<CustomData>, String> {
    // Note: FTS5 query needs to match against the workspace_id in the main table
    let sql = if category_filter.is_some() {
        "SELECT c.Id, c.WorkspaceId, c.Timestamp, c.Category, c.Key, c.Value, c.CreatedAt
         FROM CustomData c
         WHERE c.WorkspaceId = ?1 AND c.Category = ?2
         AND c.Id IN (
             SELECT rowid FROM custom_data_fts WHERE custom_data_fts MATCH ?3
         )
         ORDER BY c.Timestamp DESC
         LIMIT ?4"
    } else {
        "SELECT c.Id, c.WorkspaceId, c.Timestamp, c.Category, c.Key, c.Value, c.CreatedAt
         FROM CustomData c
         WHERE c.WorkspaceId = ?1
         AND c.Id IN (
             SELECT rowid FROM custom_data_fts WHERE custom_data_fts MATCH ?2
         )
         ORDER BY c.Timestamp DESC
         LIMIT ?3"
    };
    
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    
    let results = if let Some(cat) = category_filter {
        let query = format!("\"{}\"", query_term.replace("\"", "\"\""));
        stmt.query_map(
            params![workspace_id, cat, query, limit],
            |row| {
                let value_str: String = row.get(5)?;
                let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::Null);
                
                Ok(CustomData {
                    Id: row.get(0)?,
                    WorkspaceId: row.get(1)?,
                    Timestamp: row.get(2)?,
                    Category: row.get(3)?,
                    Key: row.get(4)?,
                    Value: value,
                    CreatedAt: row.get(6)?,
                })
            },
        ).map_err(|e| e.to_string())?
    } else {
        let query = format!("\"{}\"", query_term.replace("\"", "\"\""));
        stmt.query_map(
            params![workspace_id, query, limit],
            |row| {
                let value_str: String = row.get(5)?;
                let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::Null);
                
                Ok(CustomData {
                    Id: row.get(0)?,
                    WorkspaceId: row.get(1)?,
                    Timestamp: row.get(2)?,
                    Category: row.get(3)?,
                    Key: row.get(4)?,
                    Value: value,
                    CreatedAt: row.get(6)?,
                })
            },
        ).map_err(|e| e.to_string())?
    };
    
    let mut custom_data = Vec::new();
    for row in results {
        custom_data.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(custom_data)
}

// ============================================================================
// History Operations
// ============================================================================

/// Get context history
pub fn get_context_history(
    conn: &Connection,
    workspace_id: &str,
    args: &GetItemHistoryArgs,
) -> Result<Vec<ContextHistory>, String> {
    let table_name = match args.item_type.as_str() {
        "product_context" => "ProductContextHistory",
        "active_context" => "ActiveContextHistory",
        _ => return Err("Invalid item_type".to_string()),
    };
    
    let mut sql = format!(
        "SELECT Id, WorkspaceId, Timestamp, Version, Content, ChangeSource 
         FROM {} WHERE WorkspaceId = ?1",
        table_name
    );
    
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(workspace_id.to_string())];
    
    if let Some(version) = args.version {
        sql.push_str(" AND Version = ?");
        params_vec.push(Box::new(version));
    }
    
    if let Some(ref before) = args.before_timestamp {
        sql.push_str(" AND Timestamp < ?");
        params_vec.push(Box::new(before.clone()));
    }
    
    if let Some(ref after) = args.after_timestamp {
        sql.push_str(" AND Timestamp > ?");
        params_vec.push(Box::new(after.clone()));
    }
    
    sql.push_str(" ORDER BY Version DESC");
    
    if let Some(limit) = args.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        let content_str: String = row.get(4)?;
        let content: Value = serde_json::from_str(&content_str).unwrap_or(Value::Null);
        
        Ok(ContextHistory {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Version: row.get(3)?,
            Content: content,
            ChangeSource: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(results)
}

/// Add a context history entry
pub fn add_context_history(
    conn: &Connection,
    workspace_id: &str,
    item_type: &str,
    content: &str,
    change_source: Option<&str>,
) -> Result<i64, String> {
    let table_name = match item_type {
        "product_context" => "ProductContextHistory",
        "active_context" => "ActiveContextHistory",
        _ => return Err("Invalid item_type".to_string()),
    };
    
    // Get the next version number
    let version: i64 = conn.query_row(
        &format!("SELECT COALESCE(MAX(Version), 0) + 1 FROM {} WHERE WorkspaceId = ?1", table_name),
        params![workspace_id],
        |row| row.get(0),
    ).unwrap_or(1);
    
    let timestamp = current_timestamp();
    
    conn.execute(
        &format!(
            "INSERT INTO {} (WorkspaceId, Timestamp, Version, Content, ChangeSource)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            table_name
        ),
        params![workspace_id, timestamp, version, content, change_source],
    ).map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

// ============================================================================
// Context Link Operations
// ============================================================================

/// Create a link between two ConPort items
pub fn create_link(
    conn: &Connection,
    args: &LinkConportItemsArgs,
) -> Result<ContextLink, String> {
    let timestamp = current_timestamp();
    
    conn.execute(
        "INSERT INTO ContextLinks (WorkspaceId, Timestamp, SourceItemType, SourceItemId, TargetItemType, TargetItemId, RelationshipType, Description)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            args.workspace_id,
            timestamp,
            args.source_item_type,
            args.source_item_id,
            args.target_item_type,
            args.target_item_id,
            args.relationship_type,
            args.description,
        ],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    
    Ok(ContextLink {
        Id: id.to_string(),
        WorkspaceId: args.workspace_id.clone(),
        Timestamp: timestamp,
        SourceItemType: args.source_item_type.clone(),
        SourceItemId: args.source_item_id.clone(),
        TargetItemType: args.target_item_type.clone(),
        TargetItemId: args.target_item_id.clone(),
        RelationshipType: args.relationship_type.clone(),
        Description: args.description.clone(),
    })
}

/// Get linked items for a ConPort item
pub fn get_linked_items(
    conn: &Connection,
    workspace_id: &str,
    args: &GetLinkedItemsArgs,
) -> Result<Vec<ContextLink>, String> {
    // Get both incoming and outgoing links
    let mut sql = String::from(
        "SELECT Id, WorkspaceId, Timestamp, SourceItemType, SourceItemId, TargetItemType, TargetItemId, RelationshipType, Description 
         FROM ContextLinks WHERE WorkspaceId = ?1 
         AND (SourceItemType = ?2 AND SourceItemId = ?3 OR TargetItemType = ?2 AND TargetItemId = ?3)"
    );
    
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(workspace_id.to_string()),
        Box::new(args.item_type.clone()),
        Box::new(args.item_id.clone()),
    ];
    
    if let Some(ref rel_type) = args.relationship_type_filter {
        sql.push_str(" AND RelationshipType = ?");
        params_vec.push(Box::new(rel_type.clone()));
    }
    
    if let Some(ref item_type) = args.linked_item_type_filter {
        sql.push_str(" AND (SourceItemType = ? OR TargetItemType = ?)");
        params_vec.push(Box::new(item_type.clone()));
        params_vec.push(Box::new(item_type.clone()));
    }
    
    sql.push_str(" ORDER BY Timestamp DESC");
    
    if let Some(limit) = args.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(ContextLink {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            SourceItemType: row.get(3)?,
            SourceItemId: row.get(4)?,
            TargetItemType: row.get(5)?,
            TargetItemId: row.get(6)?,
            RelationshipType: row.get(7)?,
            Description: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(results)
}

/// Delete a link by ID
pub fn delete_link(
    conn: &Connection,
    workspace_id: &str,
    link_id: i64,
) -> Result<bool, String> {
    let rows = conn.execute(
        "DELETE FROM ContextLinks WHERE Id = ? AND WorkspaceId = ?",
        params![link_id, workspace_id],
    ).map_err(|e| e.to_string())?;
    
    Ok(rows > 0)
}

// ============================================================================
// Decision Operations
// ============================================================================

/// Log a new decision
pub fn log_decision(
    conn: &Connection,
    args: &LogDecisionArgs,
) -> Result<Decision, String> {
    let timestamp = current_timestamp();
    let tags_json = args.tags.as_ref()
        .map(|tags| serde_json::to_string(tags).unwrap_or_default())
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO decisions (WorkspaceId, Timestamp, Summary, Rationale, ImplementationDetails, Tags, CreatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            args.workspace_id,
            timestamp,
            args.summary,
            args.rationale,
            args.implementation_details,
            tags_json,
            timestamp,
        ],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(Decision {
        Id: id,
        WorkspaceId: args.workspace_id.clone(),
        Timestamp: timestamp.clone(),
        Summary: args.summary.clone(),
        Rationale: args.rationale.clone(),
        ImplementationDetails: args.implementation_details.clone(),
        Tags: Some(tags_json),
        CreatedAt: timestamp,
    })
}

/// Get decisions with filters
pub fn get_decisions(
    conn: &Connection,
    workspace_id: &str,
    args: &GetDecisionsArgs,
) -> Result<Vec<Decision>, String> {
    let mut sql = String::from(
        "SELECT Id, WorkspaceId, Timestamp, Summary, Rationale, ImplementationDetails, Tags, CreatedAt
         FROM decisions WHERE WorkspaceId = ?1"
    );

    sql.push_str(" ORDER BY Timestamp DESC");

    if let Some(limit) = args.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(Decision {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Summary: row.get(3)?,
            Rationale: row.get(4)?,
            ImplementationDetails: row.get(5)?,
            Tags: row.get(6)?,
            CreatedAt: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }

    Ok(results)
}

/// Get a single decision by ID
pub fn get_decision_by_id(
    conn: &Connection,
    workspace_id: &str,
    decision_id: i64,
) -> Result<Option<Decision>, String> {
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, Summary, Rationale, ImplementationDetails, Tags, CreatedAt
         FROM decisions WHERE Id = ?1 AND WorkspaceId = ?2"
    ).map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![decision_id, workspace_id], |row| {
        Ok(Decision {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Summary: row.get(3)?,
            Rationale: row.get(4)?,
            ImplementationDetails: row.get(5)?,
            Tags: row.get(6)?,
            CreatedAt: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    // Return the first result if exists
    if let Some(result) = rows.next() {
        Ok(Some(result.map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

/// Update a decision
pub fn update_decision(
    conn: &Connection,
    workspace_id: &str,
    args: &UpdateDecisionArgs,
) -> Result<bool, String> {
    let mut updates = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(workspace_id.to_string())];

    if let Some(ref summary) = args.summary {
        updates.push("Summary = ?");
        params_vec.push(Box::new(summary.clone()));
    }

    if let Some(ref rationale) = args.rationale {
        updates.push("Rationale = ?");
        params_vec.push(Box::new(rationale.clone()));
    }

    if let Some(ref implementation_details) = args.implementation_details {
        updates.push("ImplementationDetails = ?");
        params_vec.push(Box::new(implementation_details.clone()));
    }

    if let Some(ref tags) = args.tags {
        let tags_json = serde_json::to_string(tags).unwrap_or_default();
        updates.push("Tags = ?");
        params_vec.push(Box::new(tags_json));
    }

    if updates.is_empty() {
        return Ok(false);
    }

    // Add decision_id to params
    params_vec.push(Box::new(args.decision_id));

    let sql = format!(
        "UPDATE decisions SET {} WHERE Id = ? AND WorkspaceId = ?",
        updates.join(", ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let rows = conn.execute(&sql, params_refs.as_slice()).map_err(|e| e.to_string())?;

    Ok(rows > 0)
}

/// Delete a decision by ID
pub fn delete_decision(
    conn: &Connection,
    workspace_id: &str,
    decision_id: i64,
) -> Result<bool, String> {
    let rows = conn.execute(
        "DELETE FROM decisions WHERE Id = ? AND WorkspaceId = ?",
        params![decision_id, workspace_id],
    ).map_err(|e| e.to_string())?;

    Ok(rows > 0)
}

/// Search decisions using FTS5
pub fn search_decisions_fts(
    conn: &Connection,
    workspace_id: &str,
    query_term: &str,
    tags_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<Decision>, String> {
    let query = format!("\"{}\"", query_term.replace("\"", "\"\""));
    
    let sql = "SELECT d.Id, d.WorkspaceId, d.Timestamp, d.Summary, d.Rationale, d.ImplementationDetails, d.Tags, d.CreatedAt
               FROM decisions d
               WHERE d.WorkspaceId = ?1
               AND d.Id IN (
                   SELECT rowid FROM decisions_fts WHERE decisions_fts MATCH ?2
               )
               ORDER BY d.Timestamp DESC
               LIMIT ?3";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![workspace_id, query, limit], |row| {
        Ok(Decision {
            Id: row.get(0)?,
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Summary: row.get(3)?,
            Rationale: row.get(4)?,
            ImplementationDetails: row.get(5)?,
            Tags: row.get(6)?,
            CreatedAt: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }

    Ok(results)
}

// ============================================================================
// Activity Summary Operations
// ============================================================================

/// Get recent activity summary - queries all tables for recent items
pub fn get_recent_activity_summary(
    conn: &Connection,
    workspace_id: &str,
    hours_ago: i64,
    limit_per_type: i64,
) -> Result<RecentActivityData, String> {
    let timestamp_filter = chrono::Utc::now() - chrono::Duration::hours(hours_ago);
    let timestamp_str = timestamp_filter.to_rfc3339();

    // Get recent product context changes
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, Version, Content, ChangeSource
         FROM ProductContextHistory
         WHERE WorkspaceId = ?1 AND Timestamp > ?2
         ORDER BY Timestamp DESC LIMIT ?3"
    ).map_err(|e| e.to_string())?;

    let product_context: Vec<RecentContextHistory> = stmt.query_map(params![workspace_id, timestamp_str, limit_per_type], |row| {
        let content_str: String = row.get(4)?;
        let content: Value = serde_json::from_str(&content_str).unwrap_or(Value::Null);
        let id: i64 = row.get(0)?;
        Ok(RecentContextHistory {
            Id: id.to_string(),
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Version: row.get(3)?,
            Content: content,
            ChangeSource: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Get recent active context changes
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, Version, Content, ChangeSource
         FROM ActiveContextHistory
         WHERE WorkspaceId = ?1 AND Timestamp > ?2
         ORDER BY Timestamp DESC LIMIT ?3"
    ).map_err(|e| e.to_string())?;

    let active_context: Vec<RecentContextHistory> = stmt.query_map(params![workspace_id, timestamp_str, limit_per_type], |row| {
        let content_str: String = row.get(4)?;
        let content: Value = serde_json::from_str(&content_str).unwrap_or(Value::Null);
        let id: i64 = row.get(0)?;
        Ok(RecentContextHistory {
            Id: id.to_string(),
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Version: row.get(3)?,
            Content: content,
            ChangeSource: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Get recent progress entries
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, Status, Description, ParentId, CreatedAt, UpdatedAt
         FROM ProgressEntries
         WHERE WorkspaceId = ?1 AND Timestamp > ?2
         ORDER BY Timestamp DESC LIMIT ?3"
    ).map_err(|e| e.to_string())?;

    let progress_entries: Vec<RecentProgress> = stmt.query_map(params![workspace_id, timestamp_str, limit_per_type], |row| {
        let id: i64 = row.get(0)?;
        let parent_id: Option<i64> = row.get(5)?;
        Ok(RecentProgress {
            Id: id.to_string(),
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Status: row.get(3)?,
            Description: row.get(4)?,
            ParentId: parent_id.map(|p| p.to_string()),
            CreatedAt: row.get(6)?,
            UpdatedAt: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Get recent decisions
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, Summary, Rationale, ImplementationDetails, Tags, CreatedAt
         FROM decisions
         WHERE WorkspaceId = ?1 AND Timestamp > ?2
         ORDER BY Timestamp DESC LIMIT ?3"
    ).map_err(|e| e.to_string())?;
    
    let decisions: Vec<RecentDecision> = stmt.query_map(params![workspace_id, timestamp_str, limit_per_type], |row| {
        let id: i64 = row.get(0)?;
        Ok(RecentDecision {
            Id: id.to_string(),
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Summary: row.get(3)?,
            Rationale: row.get(4)?,
            ImplementationDetails: row.get(5)?,
            Tags: row.get(6)?,
            CreatedAt: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    // Get recent system patterns
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, Name, Description, Tags, CreatedAt
         FROM SystemPatterns
         WHERE WorkspaceId = ?1 AND Timestamp > ?2
         ORDER BY Timestamp DESC LIMIT ?3"
    ).map_err(|e| e.to_string())?;
    
    let system_patterns: Vec<RecentSystemPattern> = stmt.query_map(params![workspace_id, timestamp_str, limit_per_type], |row| {
        let id: i64 = row.get(0)?;
        let tags_str: Option<String> = row.get(5)?;
        let tags: Option<Vec<String>> = tags_str.and_then(|s| serde_json::from_str(&s).ok());
        Ok(RecentSystemPattern {
            Id: id.to_string(),
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Name: row.get(3)?,
            Description: row.get(4)?,
            Tags: tags,
            CreatedAt: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    // Get recent custom data
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, Category, Key, Value, CreatedAt
         FROM CustomData
         WHERE WorkspaceId = ?1 AND Timestamp > ?2
         ORDER BY Timestamp DESC LIMIT ?3"
    ).map_err(|e| e.to_string())?;
    
    let custom_data: Vec<RecentCustomData> = stmt.query_map(params![workspace_id, timestamp_str, limit_per_type], |row| {
        let id: i64 = row.get(0)?;
        let value_str: String = row.get(5)?;
        let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::Null);
        Ok(RecentCustomData {
            Id: id.to_string(),
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            Category: row.get(3)?,
            Key: row.get(4)?,
            Value: value,
            CreatedAt: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    // Get recent context links
    let mut stmt = conn.prepare(
        "SELECT Id, WorkspaceId, Timestamp, SourceItemType, SourceItemId, TargetItemType, TargetItemId, RelationshipType, Description
         FROM ContextLinks
         WHERE WorkspaceId = ?1 AND Timestamp > ?2
         ORDER BY Timestamp DESC LIMIT ?3"
    ).map_err(|e| e.to_string())?;
    
    let context_links: Vec<RecentContextLink> = stmt.query_map(params![workspace_id, timestamp_str, limit_per_type], |row| {
        let id: i64 = row.get(0)?;
        Ok(RecentContextLink {
            Id: id.to_string(),
            WorkspaceId: row.get(1)?,
            Timestamp: row.get(2)?,
            SourceItemType: row.get(3)?,
            SourceItemId: row.get(4)?,
            TargetItemType: row.get(5)?,
            TargetItemId: row.get(6)?,
            RelationshipType: row.get(7)?,
            Description: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(RecentActivityData {
        product_context,
        active_context,
        progress_entries,
        decisions,
        system_patterns,
        custom_data,
        context_links,
    })
}

/// Recent activity data structure (using String IDs for compatibility)
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentActivityData {
    pub product_context: Vec<RecentContextHistory>,
    pub active_context: Vec<RecentContextHistory>,
    pub progress_entries: Vec<RecentProgress>,
    pub decisions: Vec<RecentDecision>,
    pub system_patterns: Vec<RecentSystemPattern>,
    pub custom_data: Vec<RecentCustomData>,
    pub context_links: Vec<RecentContextLink>,
}

/// String-based versions for activity summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentContextHistory {
    pub Id: String,
    pub WorkspaceId: String,
    pub Timestamp: String,
    pub Version: i64,
    pub Content: Value,
    pub ChangeSource: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentProgress {
    pub Id: String,
    pub WorkspaceId: String,
    pub Timestamp: String,
    pub Status: String,
    pub Description: String,
    pub ParentId: Option<String>,
    pub CreatedAt: String,
    pub UpdatedAt: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentDecision {
    pub Id: String,
    pub WorkspaceId: String,
    pub Timestamp: String,
    pub Summary: String,
    pub Rationale: Option<String>,
    pub ImplementationDetails: Option<String>,
    pub Tags: Option<String>,
    pub CreatedAt: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentSystemPattern {
    pub Id: String,
    pub WorkspaceId: String,
    pub Timestamp: String,
    pub Name: String,
    pub Description: Option<String>,
    pub Tags: Option<Vec<String>>,
    pub CreatedAt: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentCustomData {
    pub Id: String,
    pub WorkspaceId: String,
    pub Timestamp: String,
    pub Category: String,
    pub Key: String,
    pub Value: Value,
    pub CreatedAt: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentContextLink {
    pub Id: String,
    pub WorkspaceId: String,
    pub Timestamp: String,
    pub SourceItemType: String,
    pub SourceItemId: String,
    pub TargetItemType: String,
    pub TargetItemId: String,
    pub RelationshipType: String,
    pub Description: Option<String>,
}

// ============================================================================
// FTS Query Utilities
// ============================================================================

/// Prepare an FTS query to avoid parser errors
pub fn prepare_fts_query(
    query: &str,
    allowed_columns: Option<&[&str]>,
    default_column: Option<&str>,
) -> String {
    let q = query.trim();
    if q.is_empty() {
        return q.to_string();
    }

    let has_known_prefix = if let Some(cols) = allowed_columns {
        cols.iter().any(|c| q.contains(&format!("{}:", c)))
    } else {
        false
    };

    // If colon present but not using a known prefix, treat entire query as literal
    if q.contains(':') && !has_known_prefix {
        return format!("\"{}\"", q.replace('"', "\"\""));
    }

    // If special characters that commonly break the parser are present, quote as literal
    let special_chars = ['.', '/', '\\', '"'];
    if special_chars.iter().any(|c| q.contains(*c)) && !has_known_prefix {
        if let Some(col) = default_column {
            return format!("{}:\"{}\"", col, q.replace('"', "\"\""));
        }
        return format!("\"{}\"", q.replace('"', "\"\""));
    }

    q.to_string()
}
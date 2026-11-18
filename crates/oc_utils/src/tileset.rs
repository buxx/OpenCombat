pub fn extract<I, K>(
    mapping: Vec<(I, K)>,
    tileset: &tiled::Tileset,
) -> Result<Vec<(I, u32)>, String>
where
    K: Eq + std::hash::Hash + ToString,
    I: std::fmt::Debug + Clone,
{
    mapping
        .iter()
        .map(|(i, k)| {
            match tileset.tiles().find(|(_, tile)| {
                tile.properties.get("ID") == Some(&tiled::PropertyValue::StringValue(k.to_string()))
            }) {
                Some((id, _)) => Ok((i.clone(), id)),
                None => Err(format!("No found for {i:?}")),
            }
        })
        .collect::<Result<Vec<(I, u32)>, String>>()
}

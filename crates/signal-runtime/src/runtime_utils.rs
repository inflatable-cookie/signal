pub(crate) fn sanitize_asset_id(asset_id: &str) -> String {
    asset_id
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}

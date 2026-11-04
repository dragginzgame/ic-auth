use super::*;

#[test]
fn scope_narrowing_matches_set_inclusion_for_sparse_and_unordered_requests() {
    for allowed in 0_u32..64 {
        let ceiling: Vec<_> = (0..6)
            .filter(|bit| allowed & (1 << bit) != 0)
            .map(|bit| format!("scope{bit}"))
            .collect();
        for wanted in 0_u32..64 {
            let mut requested: Vec<_> = (0..6)
                .filter(|bit| wanted & (1 << bit) != 0)
                .map(|bit| format!("scope{bit}"))
                .collect();
            requested.reverse();
            requested.extend(requested.clone());
            assert_eq!(
                scopes_are_subset(&requested, &ceiling),
                wanted & !allowed == 0
            );
        }
    }
}

#[test]
fn grant_narrowing_keeps_role_boundaries_and_missing_scope_refusals() {
    let grant = |role: &str, scopes: &[&str]| DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: scopes.iter().map(|scope| scope.parse().unwrap()).collect(),
    };
    let ceiling = [grant("a", &["read", "write"]), grant("c", &["read"])];
    for (requested, expected) in [
        (vec![], true),
        (vec![grant("a", &["write"]), grant("c", &["read"])], true),
        (vec![grant("b", &["read"])], false),
        (vec![grant("d", &["read"])], false),
        (vec![grant("c", &["write"])], false),
        (vec![grant("a", &["read", "read_all"])], false),
    ] {
        assert_eq!(grants_are_subset(&requested, &ceiling), expected);
    }
}

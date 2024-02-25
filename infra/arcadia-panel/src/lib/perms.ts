/*
```rust 

pub fn has_perm(perms: &Vec<String>, perm: &str) -> bool {
    let mut perm_split = perm.split('.').collect::<Vec<&str>>();

    if perm_split.len() < 2 {
        // Then assume its a global permission on the namespace
        perm_split = vec![perm, "*"];
    }

    let perm_namespace = perm_split[0];
    let perm_name = perm_split[1];

    let mut has_perm = None;
    let mut has_negator = false;
    for user_perm in perms {
        if user_perm == "global.*" {
            // Special case
            return true;
        }

        let mut user_perm_split = user_perm.split('.').collect::<Vec<&str>>();

        if user_perm_split.len() < 2 {
            // Then assume its a global permission
            user_perm_split = vec![user_perm, "*"];
        }

        let mut user_perm_namespace = user_perm_split[0];
        let user_perm_name = user_perm_split[1];

        if user_perm.starts_with('~') {
            // Strip the ~ from namespace to check it
            user_perm_namespace = user_perm_namespace.trim_start_matches('~');
        }

        if (user_perm_namespace == perm_namespace
            || user_perm_namespace == "global")
            && (user_perm_name == "*" || user_perm_name == perm_name)
        {
            // We have to check for all negator
            has_perm = Some(user_perm_split);

            if user_perm.starts_with('~') {
                has_negator = true; // While we can optimize here by returning false, we may want to add more negation systems in the future
            }
        }
    }

    has_perm.is_some() && !has_negator
}

/// Builds a permission string from a namespace and permission
pub fn build(namespace: &str, perm: &str) -> String {
    format!("{}.{}", namespace, perm)
}
```
*/

/**
 * Given a resolved set of perms, check if a given namespace.permission is allowed
 *
 * See https://github.com/InfinityBotList/kittycat/blob/main/src/perms.rs
 *
 * Should be equivalent to the above Rust code
 */
export const hasPerm = (perms: string[], perm: string): boolean => {
	let perm_split = perm.split('.');

	if (perm_split.length < 2) {
		// Then assume its a global permission on the namespace
		perm_split = [perm, '*'];
	}

	const perm_namespace = perm_split[0];
	const perm_name = perm_split[1];

	let has_perm = null;
	let has_negator = false;
	for (const user_perm of perms) {
		if (user_perm == 'global.*') {
			// Special case
			return true;
		}

		let user_perm_split = user_perm.split('.');

		if (user_perm_split.length < 2) {
			// Then assume its a global permission
			user_perm_split = [user_perm, '*'];
		}

		let user_perm_namespace = user_perm_split[0];
		const user_perm_name = user_perm_split[1];

		if (user_perm.startsWith('~')) {
			// Strip the ~ from namespace to check it
			// DIFFERENCE FROM RUST CODE: substring is used instead of trim_start_matches*
			user_perm_namespace = user_perm_namespace.substring(1);
		}

		if (
			(user_perm_namespace == perm_namespace || user_perm_namespace == 'global') &&
			(user_perm_name == '*' || user_perm_name == perm_name)
		) {
			// We have to check for all negator
			has_perm = user_perm_split;

			if (user_perm.startsWith('~')) {
				has_negator = true; // While we can optimize here by returning false, we may want to add more negation systems in the future
			}
		}
	}

	return has_perm != null && !has_negator;
};

/**
 * Builds a permission string from a namespace and permission
 */
export const build = (namespace: string, perm: string): string => {
	return `${namespace}.${perm}`;
};

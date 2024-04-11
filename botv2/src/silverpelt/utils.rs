/// From name_split, construct a list of all permutations of the command name from the root till the end
///
/// E.g: If subcommand is `limits hit`, then `limits` and `limits hit` will be constructed
///     as the list of commands to check
/// E.g 2: If subcommand is `limits hit add`, then `limits`, `limits hit` and `limits hit add`
///     will be constructed as the list of commands to check
pub fn permute_command_names(name: &str) -> Vec<String> {
    // Check if subcommand by splitting the command name
    let name_split = name.split(' ').collect::<Vec<&str>>();

    let mut commands_to_check = Vec::new();

    for i in 0..name_split.len() {
        let mut command = String::new();

        for (j, cmd) in name_split.iter().enumerate().take(i + 1) {
            command += cmd;

            if j != i {
                command += " ";
            }
        }

        commands_to_check.push(command);
    }

    commands_to_check
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permute_command_names() {
        assert_eq!(permute_command_names(""), vec![""]);
        assert_eq!(permute_command_names("limits"), vec!["limits"]);
        assert_eq!(
            permute_command_names("limits hit"),
            vec!["limits", "limits hit"]
        );
        assert_eq!(
            permute_command_names("limits hit add"),
            vec!["limits", "limits hit", "limits hit add"]
        );
    }
}

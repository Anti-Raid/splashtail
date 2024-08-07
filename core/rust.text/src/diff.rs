/// Defines the contents of a changeset
/// Changesets will be delivered in order of appearance in the original string
/// Sequences of the same kind will be grouped into one Difference
#[derive(PartialEq, Debug)]
pub enum Difference {
    /// Sequences that are the same
    Same(String),
    /// Sequences that are an addition (don't appear in the first string)
    Add(String),
    /// Sequences that are a removal (don't appear in the second string)
    Rem(String),
}

pub fn diff(orig: &str, edit: &str, split: &str) -> (i32, Vec<Difference>) {
    let (dist, common) = lcs(orig, edit, split);
    (dist, merge(orig, edit, &common, split))
}

// finds the longest common subsequences
// outputs the edit distance and a string containing
// all chars both inputs have in common
#[allow(non_snake_case)]
pub fn lcs(orig: &str, edit: &str, split: &str) -> (i32, String) {
    // make list by custom splits
    let a: Vec<&str> = orig.split(split).collect();
    let b: Vec<&str> = edit.split(split).collect();

    let N = a.len() as i32;
    let M = b.len() as i32;

    let MAX = N + M;

    let mut v: Vec<i32> = (-MAX..MAX).collect();

    // container to hold common subsequence
    let mut common = String::new();

    v[1] = 0;

    // iterate over D = "edit steps"
    for D in 0..MAX {
        let mut max = 0;
        let mut max_snake: Box<String> = Box::new("".to_string());

        // TODO replace with
        // for k in (-D..D+1).step_by(2) {
        // once it's stable

        let mut k = -D;

        while k < D + 1 {
            let mut snake = String::new();

            let mut x;

            let index = (MAX + k - 1) as usize;
            if k == -D || k != D && v[index - 1] < v[index + 1] {
                x = v[index + 1];
            } else {
                x = v[index - 1] + 1;
            }

            let mut y = x - k;

            while x < N && y < M && a[x as usize] == b[y as usize] {
                if !snake.is_empty() {
                    // add back the splits that were taken away
                    snake.push_str(split);
                }
                snake.push_str(a[x as usize]);
                x += 1;
                y += 1;
            }

            v[index] = x;

            if x > max {
                max = x;
                max_snake = Box::new(snake);
            }

            if x >= N && y >= M {
                // add last max_snake
                if max_snake.len() > 0 {
                    if !common.is_empty() {
                        // add back the splits that were taken away
                        common.push_str(split);
                    }
                    common.push_str(&max_snake);
                } else {
                    common.push_str(split);
                }
                return (D, common);
            }
            k += 2;
        }

        if !max_snake.is_empty() {
            if !common.is_empty() {
                // add back the splits that were taken away
                common.push_str(split);
            }
            common.push_str(&max_snake);
        }
    }

    // both strings don't match at all
    (MAX, "".to_string())
}

// merges the changes from two strings, given a common substring
pub fn merge(orig: &str, edit: &str, common: &str, split: &str) -> Vec<Difference> {
    let mut ret = Vec::new();

    let mut a = orig.split(split);
    let mut b = edit.split(split);

    let mut same = String::new();
    for c in common.split(split) {
        let mut add = String::new();
        let mut rem = String::new();

        let mut x = a.next();
        while x.is_some() && Some(c) != x {
            if !rem.is_empty() {
                rem.push_str(split);
            }
            rem.push_str(x.unwrap());
            x = a.next();
        }

        let mut y = b.next();
        while y.is_some() && Some(c) != y {
            if !add.is_empty() {
                add.push_str(split);
            }
            add.push_str(y.unwrap());
            y = b.next();
        }

        if !add.is_empty() || !rem.is_empty() {
            ret.push(Difference::Same(same.clone()));
            same.clear();
        }

        if !rem.is_empty() {
            ret.push(Difference::Rem(rem.clone()));
        }

        if !add.is_empty() {
            ret.push(Difference::Add(add.clone()));
        }

        if !same.is_empty() {
            same.push_str(split);
        }
        same.push_str(c);
    }
    if !same.is_empty() {
        ret.push(Difference::Same(same.clone()));
    }

    // TODO avoid duplication

    let mut rem = String::new();

    for x in a {
        if !rem.is_empty() {
            rem.push_str(split);
        }
        rem.push_str(x);
    }
    if !rem.is_empty() {
        ret.push(Difference::Rem(rem.clone()));
    }

    let mut add = String::new();
    for y in b {
        if !add.is_empty() {
            add.push_str(split);
        }
        add.push_str(y);
    }
    if !add.is_empty() {
        ret.push(Difference::Add(add.clone()));
    }

    ret
}

#[allow(dead_code)]
/// Returns a colorful visual representation of the diff.
pub fn return_diff_colorful(orig: &str, edit: &str, split: &str) -> String {
    let (_, changeset) = diff(orig, edit, split);
    let mut ret = String::new();

    for seq in changeset {
        match seq {
            Difference::Same(ref x) => {
                ret.push_str(x);
                ret.push_str(split);
            }
            Difference::Add(ref x) => {
                ret.push_str("\x1B[92m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push_str(split);
            }
            Difference::Rem(ref x) => {
                ret.push_str("\x1B[91m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push_str(split);
            }
        }
    }

    ret
}

#[test]
fn test_lcs() {
    assert_eq!(lcs("test", "tost", ""), (2, "tst".to_string()));
    assert_eq!(lcs("test", "test", ""), (0, "test".to_string()));

    assert_eq!(lcs("test", "test", " "), (0, "test".to_string()));

    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown dog leaps over the lazy cat",
            ""
        ),
        (16, "The quick brown o ps over the lazy ".to_string())
    );
    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown dog leaps over the lazy cat",
            " "
        ),
        (6, "The quick brown over the lazy ".to_string())
    );

    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown dog leaps over the lazy cat",
            "\n"
        ),
        (2, "".to_string())
    );
    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown fox jumps over the lazy dog",
            "\n"
        ),
        (0, "The quick brown fox jumps over the lazy dog".to_string())
    );
}

#[test]
fn test_merge() {
    assert_eq!(
        merge("testa", "tost", "tst", ""),
        vec![
            Difference::Same("t".to_string()),
            Difference::Rem("e".to_string()),
            Difference::Add("o".to_string()),
            Difference::Same("st".to_string()),
            Difference::Rem("a".to_string()),
        ]
    );
}

#[test]
fn test_diff() {
    let text1 = "Roses are red, violets are blue,\n\
                 I wrote this library,\n\
                 just for you.\n\
                 (It's true).";

    let text2 = "Roses are red, violets are blue,\n\
                 I wrote this documentation,\n\
                 just for you.\n\
                 (It's quite true).";

    let (dist, changeset) = diff(text1, text2, "\n");

    assert_eq!(dist, 4);

    assert_eq!(
        changeset,
        vec![
            Difference::Same("Roses are red, violets are blue,".to_string()),
            Difference::Rem("I wrote this library,".to_string()),
            Difference::Add("I wrote this documentation,".to_string()),
            Difference::Same("just for you.".to_string()),
            Difference::Rem("(It's true).".to_string()),
            Difference::Add("(It's quite true).".to_string())
        ]
    );
}

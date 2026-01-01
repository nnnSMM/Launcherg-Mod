pub struct Distance {
    a: Vec<char>,
    b: Vec<char>,
    m: usize,
    n: usize, // m <= n, m is a length, n is b length
}

impl Distance {
    pub fn new(a: &str, b: &str) -> Self {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let (m, n) = (a.len(), b.len());

        if m > n {
            return Distance {
                a: b,
                b: a,
                m: n,
                n: m,
            };
        }

        Distance { a, b, m, n }
    }

    pub fn onp(&self) -> usize {
        let offset: isize = (self.m as isize) + 1;
        let delta: isize = (self.n as isize) - (self.m as isize);
        let mut fp = vec![-1; self.m + self.n + 3];

        let mut p: isize = 0;
        loop {
            // -p <= k <= delta - 1
            for k in (-p)..=(delta - 1) {
                fp[(k + offset) as usize] = self.snake(
                    k,
                    (fp[(k - 1 + offset) as usize] + 1).max(fp[(k + 1 + offset) as usize]),
                );
            }
            // delta + 1 <= k <= delta + p
            for k in ((delta + 1)..=(delta + p)).rev() {
                fp[(k + offset) as usize] = self.snake(
                    k,
                    (fp[(k - 1 + offset) as usize] + 1).max(fp[(k + 1 + offset) as usize]),
                );
            }
            // delta == k
            fp[(delta + offset) as usize] = self.snake(
                delta,
                (fp[(delta - 1 + offset) as usize] + 1).max(fp[(delta + 1 + offset) as usize]),
            );
            if fp[(delta + offset) as usize] == (self.n as isize) {
                return (delta + 2 * p) as usize;
            }
            p += 1;
        }
    }

    fn snake(&self, k: isize, y: isize) -> isize {
        let mut x = y - k;
        let mut y = y;
        while x < self.m as isize && y < self.n as isize && self.a[x as usize] == self.b[y as usize]
        {
            x += 1;
            y += 1;
        }
        y
    }
}

pub fn get_comparable_distance(a: &str, b: &str) -> f32 {
    let distance = Distance::new(&a, &b);
    let distance_value = distance.onp();

    1.0 - (distance_value as f32 / a.len().max(b.len()) as f32)
}

pub fn find_nearest<'a>(
    key: &'a String,
    list: &'a Vec<(String, String)>,
) -> (Option<&'a String>, f32) {
    let mut max_distance = 0.0;
    let mut max_distance_value = None;
    for (comp_key, comp_value) in list {
        let distance = get_comparable_distance(key, comp_key);
        if distance > max_distance {
            max_distance = distance;
            max_distance_value = Some(comp_value);
        }
    }
    (max_distance_value, max_distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onp() {
        let cases = vec![("abc", "abcdef", 3), ("abc", "ab", 1), ("abc", "abc", 0)];

        for (a, b, expected) in cases {
            let distance = Distance::new(a, b);
            let result = distance.onp();
            assert_eq!(result, expected, "Failed on values a:{}, b:{}", a, b);
        }
    }

    #[test]
    fn test_onp_empty_strings() {
        // 空文字のテスト
        assert_eq!(Distance::new("", "").onp(), 0);
        assert_eq!(Distance::new("", "abc").onp(), 3);
        assert_eq!(Distance::new("abc", "").onp(), 3);
    }

    #[test]
    fn test_onp_japanese_characters() {
        // 日本語文字列のテスト
        // 「こんにちは」→「こんばんは」: 「にち」→「ばん」で4つの編集
        let d = Distance::new("こんにちは", "こんばんは");
        assert_eq!(d.onp(), 4);
    }

    #[test]
    fn test_get_comparable_distance() {
        // 完全一致は1.0
        let same = get_comparable_distance("hello", "hello");
        assert!((same - 1.0).abs() < 0.001, "Expected 1.0, got {}", same);

        // 「abc」→「abcd」は1文字追加で距離1、長さ4で 1.0 - 1/4 = 0.75
        let partial = get_comparable_distance("abc", "abcd");
        assert!(
            (partial - 0.75).abs() < 0.01,
            "Expected 0.75, got {}",
            partial
        );

        // 類似文字列は正の類似度を持つ
        let similar = get_comparable_distance("hello", "hallo");
        assert!(similar > 0.0, "Expected >0, got {}", similar);

        // 完全に異なる文字列は低い（または負の）類似度
        let diff = get_comparable_distance("abc", "xyz");
        assert!(diff < 0.5, "Expected <0.5, got {}", diff);
    }

    #[test]
    fn test_find_nearest() {
        let list = vec![
            ("apple".to_string(), "1".to_string()),
            ("banana".to_string(), "2".to_string()),
            ("apricot".to_string(), "3".to_string()),
        ];

        let key = "aple".to_string(); // "apple"のタイポ
        let (result, score) = find_nearest(&key, &list);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "1"); // "apple"が最も近い
        assert!(score > 0.5);
    }

    #[test]
    fn test_find_nearest_empty_list() {
        let list: Vec<(String, String)> = vec![];
        let key = "test".to_string();
        let (result, score) = find_nearest(&key, &list);

        assert!(result.is_none());
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_find_nearest_single_element() {
        let list = vec![("hello".to_string(), "1".to_string())];
        let key = "hello".to_string();
        let (result, score) = find_nearest(&key, &list);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "1");
        assert!((score - 1.0).abs() < 0.001); // 完全一致なのでスコア1.0
    }

    #[test]
    fn test_find_nearest_exact_match() {
        let list = vec![
            ("test".to_string(), "A".to_string()),
            ("testing".to_string(), "B".to_string()),
            ("best".to_string(), "C".to_string()),
        ];
        let key = "test".to_string();
        let (result, score) = find_nearest(&key, &list);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "A"); // 完全一致
        assert!((score - 1.0).abs() < 0.001);
    }
}

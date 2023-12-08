/// Maintaining the sentence words sequence in a ring buffer with a certain number of words.
/// The sequence starts with a word and ends with word, with non-words (e.g. punctuation or space) in between.
pub struct SentenceRingBuffer {
    /// (word, true - a real word, false - non-word)
    words: Vec<(String, bool)>,
    head: usize,
    tail: usize,
    /// Number of words in the ring buffer.
    num_words: usize,
    /// Number of words in a sentence.
    sentence_words_num: usize,
}

impl SentenceRingBuffer {
    pub fn new(sentence_words_num: usize) -> Self {
        Self {
            words: vec![(String::new(), false); sentence_words_num * 2],
            head: 0,
            tail: 0,
            num_words: 0,
            sentence_words_num,
        }
    }

    fn next_index(&self, index: usize) -> usize {
        (index + 1) % self.words.len()
    }

    fn create_current_sentence(&self) -> String {
        let mut sentence = String::new();

        let mut scan_index = self.head;
        while scan_index != self.tail {
            sentence.push_str(&self.words[scan_index].0);
            scan_index = self.next_index(scan_index);
        }

        sentence
    }

    /// Add a word into the ring buffer.
    /// If there are enough words for a sentence in the ring buffer, the sentence is returned.
    pub fn add(&mut self, word: String, is_word: bool) -> Option<String> {
        let mut sentence: Option<String> = None;
        if is_word {
            // A new word added
            if self.num_words == self.sentence_words_num {
                // There are already enough words in the ring buffer.
                // Pop the last words.
                self.head = self.next_index(self.head);
                // Pop the last non-word.
                if self.head != self.tail {
                    let head_word = &self.words[self.head];
                    if !head_word.1 {
                        self.head = self.next_index(self.head);
                    }
                }
            } else {
                // There are not enough words in the ring buffer. Just simply adds the word.
                self.num_words += 1;
            }
            self.words[self.tail] = (word, is_word);
            self.tail = self.next_index(self.tail);

            if self.num_words == self.sentence_words_num {
                sentence = Some(self.create_current_sentence());
            }
        } else if self.head != self.tail {
            // Only add non-word if there are words in the ring buffer.
            let tail_word = if self.tail == 0 {
                self.words.last_mut().unwrap()
            } else {
                &mut self.words[self.tail - 1]
            };
            if tail_word.1 {
                // Push the non-word if the last is a word.
                self.words[self.tail] = (word, is_word);
                self.tail = self.next_index(self.tail);
            } else {
                // Concat the non-word if the last is a non-word.
                tail_word.0.push_str(&word);
            }
        }
        sentence
    }
}

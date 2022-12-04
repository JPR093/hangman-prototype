use std::collections::HashMap;
use anyhow::{Result, anyhow, Ok};

fn main() {
    let dictionary_path = String::from("dictionary.txt");
    let dictionary = 
        DictionaryData::new_with_file_location(&dictionary_path)
        .unwrap();
    let word_to_guess = String::from("intrepidness");
    let mut game = 
        TurnPertinentInfo::new_with_dictionary_data_and_word(&dictionary, &word_to_guess)
            .unwrap();

    while game.turn < 30 && !game.word_clues.iter_mut().all(|possible_char| matches!(possible_char, Some(_)))  {
        
        for i in 0..word_to_guess.len() {
            if let Some(ch) = game.word_clues[i] {
                print!("{}", ch);
            }
            else {
                print!("_");
            }
        }
        println!("");
        next_turn(&mut game).unwrap()
    }
    for i in 0..word_to_guess.len() {
        if let Some(ch) = game.word_clues[i] {
            print!("{}", ch);
        }
        else {
            print!("_");
        }
    }

    
}

// Each Vec<String> holds the strings of a given length
// The strings of len n, are at index n - 1
struct DictionaryData([Vec<String>; 50]);

impl DictionaryData {
    fn new_with_file_location(dict_file_location: &str) -> Result<DictionaryData> {
        let dictionary = std::fs::read_to_string(dict_file_location)?;
        {
            // Doing some semblance of data validation here
            let dictionary = dictionary.lines().collect::<Vec<&str>>();
            for index in 0..dictionary.len() {
                // Here chech that everything in a given line is ascii lowercase
                // Each given line will be a word in my dictionary if this is true(and the next condition)
                if dictionary[index]
                .chars()
                .any(|ch| !ch.is_ascii_lowercase())
                {
                    return Err(anyhow!("Word: {} at index: {} is not ascii lowercase", dictionary[index], index));
                }
                // Doing bounds validation for later use in this same function
                if dictionary[index].len() > 50 {
                    return Err(anyhow!("Word: {} at index: {} is more than 50 characters long, we don't do that here", dictionary[index], index));
                }
            }
        }
        let mut dict_data = DictionaryData(std::array::from_fn(|_| vec![]));

        for word in dictionary.lines() {
            dict_data.0[word.len() - 1]
                .push(word.to_owned())
        }

        Ok(dict_data)
    }
}

#[derive(Debug)]
struct TurnPertinentInfo<'a> {
    // Starts at 0, equal to the amount of guesses taken
    turn: usize,
    // Keeps track of amount of unsuccesful guesses
    failed_attempts: usize,
    // We know this one
    word: Vec<char>,
    // Keeps track of chars that have already been guessed
    unattempted_chars: Vec<char>,
    // It's none if I do not know the char in that spot
    // It's some and the respective char if I already know it
    word_clues: Vec<Option<char>>,
    // keeps track of words that are still possible to be "word" field
    // when I detect a word can no longer be "word", when I attempt a char
    // I remove it from the vector
    pertinent_words: Vec<&'a str>,
}

impl TurnPertinentInfo<'_> {
    fn new_with_dictionary_data_and_word<'a>(dict_data: &'a DictionaryData, word: &str) -> Result<TurnPertinentInfo<'a>> {
        // Will only care for the parts of the dictionary that have the words that correspond to the length
        // of my desired word
        let pertinent_words = dict_data.0[word.len() - 1]
            .iter()
            .map(|word| word.as_str())
            .collect::<Vec<&str>>();

        if !pertinent_words.contains(&word) {
            return Err(anyhow!("Attemted word: {} is not in dictionary", &word));
        }

        Ok(TurnPertinentInfo {
            turn: 0,
            failed_attempts: 0,
            word: word.chars().collect(),
            unattempted_chars: ('a'..='z').collect::<Vec<char>>(),
            word_clues: vec![None; word.len()],
            pertinent_words
        })
    }
}

fn best_char(current_turn: &TurnPertinentInfo) -> char {

    let capacity: usize = ('a'..='z').count();
    
    // It counts in how many of the possible words, the char is in
    // The more words it is in the better an idea it is to guess that one
    // char in the next turn
    let mut char_in_n_words: HashMap<char, usize> = HashMap::with_capacity(capacity);

    for &ch in current_turn.unattempted_chars.iter() {
        char_in_n_words.insert(ch, 0);
    }

    // this is to help keep track for each given word, which chars does it have
    let mut char_is_in_given_word: HashMap<char, bool> = HashMap::with_capacity(capacity);

    for &ch in current_turn.unattempted_chars.iter() {
        char_is_in_given_word.insert(ch, false);
    }
    // This previous two hashmaps have the same keys

    for word in current_turn.pertinent_words.iter() {
        for ch in word.chars() {
            if let Some(is_in_word) = char_is_in_given_word.get_mut(&ch) {
                *is_in_word = true;
            }
        }
        for (ch,count) in char_in_n_words.iter_mut() {
            if *char_is_in_given_word.get(&ch).unwrap() { //It's fine to unwrap because both hashmaps have the same keys
                *count += 1
            }
        }
        //reset for next word
        char_is_in_given_word
            .values_mut()
            .for_each(|attempted | *attempted = false);
    }

    *char_in_n_words
        .keys()
        .max_by_key(|&ch| char_in_n_words.get(ch))
        .unwrap()
}

// I need to update all the fields except for word
fn next_turn(previous_turn: &mut TurnPertinentInfo) -> Result<()> {

    // If word has already been guessed shoot up an error
    if previous_turn.word_clues
        .iter_mut()
        .all(|possible_char| matches!(possible_char, Some(_))) 
    {
        return Err(anyhow!("Word has already been guessed bonobo"));
    }


    // all words in this context(this function) have the same length
    let word_len = previous_turn.word.len();
    // turn updated here
    previous_turn.turn += 1;

    let char_to_guess = best_char(previous_turn);

    // unattempted chars updated here
    for i in 0..previous_turn.unattempted_chars.len() {
        if previous_turn.unattempted_chars[i] == char_to_guess {
            previous_turn.unattempted_chars.swap_remove(i);
            break;
        }
    }

    // changes to true if we now know the char at the given word char spot
    let new_info = {
        let mut new_info = vec![false; word_len];
        // word_clues updated here
        for i in 0..word_len {
            if previous_turn.word[i] == char_to_guess {
                previous_turn.word_clues[i] = Some(char_to_guess);
                new_info[i] = true;
            }
        }
        new_info
    };

    // amount of failed attempts updated here
    if new_info.iter().any(|&char_changed| char_changed) {
        previous_turn.failed_attempts += 1;
    }

    {
        // pertinent words updated here
        let mut word_index = 0;
        while word_index < previous_turn.pertinent_words.len() {
            if should_be_discarded(&previous_turn.pertinent_words[word_index], &new_info, char_to_guess)
                .unwrap()
            {
                previous_turn.pertinent_words.swap_remove(word_index);
            }
            else {
                word_index += 1;
            }
        }
    }

    fn should_be_discarded(word: &str, chars_changed: &[bool], char_guessed: char) -> Result<bool> {
        let mut discard = Ok(false);

        // data validation that is unnecessary in the usecase
        if word.len() != chars_changed.len() {
            return Err(anyhow!("Call me with sensical arguments bonobo"));
        }

        if chars_changed.iter().any(|&char_changed| char_changed) { //guess sucessful
            for i in 0..chars_changed.len() {
                if chars_changed[i] && word.chars().nth(i).unwrap() != char_guessed {
                    discard = Ok(true);
                    break;
                }
            }
        }
        else { //guess not succesful
            for ch in word.chars() {
                if ch == char_guessed {
                    discard = Ok(true);
                    break;
                }
            }
        }
        discard
    }

    Ok(())
}

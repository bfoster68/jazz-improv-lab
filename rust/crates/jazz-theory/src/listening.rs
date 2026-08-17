//! The listening curriculum: who to study and what repertoire to learn.
//!
//! The language is passed down by ear, so this is not decoration — it is the part
//! of the method that says *stop reading and go listen*. Facts here were checked
//! against sources; the tests below guard the ones that are easy to get wrong
//! (composer credits especially, since "Take the A Train" is routinely misattributed
//! to Ellington when Strayhorn wrote it).

/// A player worth living with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Master {
    pub name: &'static str,
    /// Primary instrument.
    pub instrument: &'static str,
    /// Active era, and style where it clarifies.
    pub era: &'static str,
    /// What to listen *for* — the point is to train the ear, not to memorise trivia.
    pub listen_for: &'static str,
    /// Year and place of birth.
    pub born: &'static str,
    /// Year of death; empty while the master is still with us.
    pub died: &'static str,
    /// A short introduction — who this was and why it matters.
    pub bio: &'static str,
    /// Notable events, personal and musical, in chronological order: (year, event).
    pub life: &'static [(&'static str, &'static str)],
}

pub const MASTERS: &[Master] = &[
    Master { name: "Louis Armstrong", instrument: "Trumpet / voice", era: "1920s-70s",
        listen_for: "The first great soloist - joyous swing, singing phrases, pure storytelling.",
        born: "1901, New Orleans", died: "1971",
        bio: "The first great jazz soloist, and still the most joyous. Armstrong turned an \
              ensemble music into a soloist's art, invented the vocabulary of swing phrasing, \
              and carried it from the streets of New Orleans to every stage on earth.",
        life: &[
            ("1901", "Born in one of the poorest quarters of New Orleans; sings on corners for coins as a boy."),
            ("1913", "Sent to the Colored Waif's Home for firing a pistol on New Year's Eve - and is handed a cornet there."),
            ("1922", "Summoned north by his mentor King Oliver to join the Creole Jazz Band in Chicago."),
            ("1925", "Begins the Hot Five and Hot Seven records - the foundation documents of the jazz solo."),
            ("1926", "Drops the paper mid-take on \"Heebie Jeebies\" and scats instead; popularises wordless singing."),
            ("1928", "Records \"West End Blues\"; its opening cadenza announces jazz as an art music."),
            ("1957", "Publicly blasts President Eisenhower over Little Rock - risking his career to say it."),
            ("1964", "\"Hello, Dolly!\" knocks the Beatles off the top of the charts; at 62 he is the oldest artist ever to do it."),
            ("1971", "Dies at home in Queens, days after playing his last engagement."),
        ] },
    Master { name: "Big Joe Turner", instrument: "Blues vocals", era: "1930s-80s",
        listen_for: "The blues-shouting roots under all of jazz - shuffle, call-and-response, feeling.",
        born: "1911, Kansas City", died: "1985",
        bio: "The Boss of the Blues. A singing bartender from Kansas City whose shout could fill \
              a hall without a microphone, Turner bridged three musics: he powered the \
              boogie-woogie craze, helped invent rhythm and blues, and lit the fuse of rock and roll.",
        life: &[
            ("1911", "Born in Kansas City; sings in the street after his father is killed by a train."),
            ("1930s", "Tends bar and shouts the blues with pianist Pete Johnson in Kansas City's wide-open clubs."),
            ("1938", "Steals the show at Carnegie Hall's \"From Spirituals to Swing\"; \"Roll 'Em Pete\" sets off the boogie-woogie craze."),
            ("1951", "Signs with Atlantic Records and becomes a rhythm-and-blues star with \"Chains of Love\"."),
            ("1954", "\"Shake, Rattle and Roll\" - the record that hands the blues to rock and roll."),
            ("1983", "Wins his place in the Blues Foundation Hall of Fame while still singing nightly."),
            ("1985", "Dies in California; inducted into the Rock and Roll Hall of Fame two years later."),
        ] },
    Master { name: "Charlie Parker", instrument: "Alto sax", era: "1940s-50s (bebop)",
        listen_for: "Blazing eighth-note lines, chromatic enclosures, the blues inside everything.",
        born: "1920, Kansas City", died: "1955",
        bio: "Bird. The central genius of bebop, who rebuilt the jazz vocabulary - the higher \
              chord tones, the chromatic approach, the headlong eighth-note line - and whose \
              brilliance and self-destruction became jazz's defining cautionary tale.",
        life: &[
            ("1920", "Born in Kansas City; obsessed with the saxophone by his early teens."),
            ("1936", "Humiliated at a Reno Club jam session when Jo Jones gongs him off with a cymbal - he answers with two years of furious practice."),
            ("1936", "A car crash the same winter brings morphine, and with it the addiction that shadows his whole life."),
            ("1939", "In New York, working over \"Cherokee\", he finds it: the higher intervals of the chords, and bebop is born."),
            ("1945", "Records \"Ko-Ko\" and the first great bebop sides with Dizzy Gillespie."),
            ("1946", "Collapses in California and spends six months at Camarillo State Hospital."),
            ("1949", "Birdland - the club named in his honour - opens on Broadway; records with strings, his own favourite of his records."),
            ("1954", "His daughter Pree dies at two; the loss breaks something that never mends."),
            ("1955", "Dies at 34 in the Hotel Stanhope; the coroner guesses his age at over 50."),
        ] },
    Master { name: "Dizzy Gillespie", instrument: "Trumpet", era: "1930s-90s (bebop / Afro-Cuban)",
        listen_for: "Harmonic daring, high-register fire, and the birth of Afro-Cuban jazz.",
        born: "1917, Cheraw, South Carolina", died: "1993",
        bio: "Bebop's co-founder and its great teacher. Where Parker burned, Gillespie built: \
              he codified the new harmony, trained two generations of players, and married \
              jazz to Cuban rhythm - then spent four decades as its happiest ambassador.",
        life: &[
            ("1917", "Born in Cheraw, South Carolina, the ninth child of a bricklayer bandleader; teaches himself trumpet."),
            ("1941", "Fired from Cab Calloway's band after the famous spitball row - and dives into the after-hours laboratories of bebop."),
            ("1945", "With Charlie Parker, records the manifesto sides of bebop: \"Salt Peanuts\", \"Hot House\", \"Groovin' High\"."),
            ("1947", "Hires Cuban conguero Chano Pozo; \"Manteca\" fuses jazz and Afro-Cuban music for good."),
            ("1948", "Pozo is murdered in Harlem; Gillespie carries the Afro-Cuban flame on alone."),
            ("1953", "A dancer falls on his trumpet and bends the bell skyward - he likes the sound, and it becomes his trademark."),
            ("1956", "Leads the first U.S. State Department jazz tour: bebop as foreign policy."),
            ("1964", "Runs a satirical campaign for President, promising to rename the White House the Blues House."),
            ("1988", "Forms the United Nation Orchestra, a last big band of proteges from across the Americas."),
            ("1993", "Dies in New Jersey, honoured on every continent he toured."),
        ] },
    Master { name: "Thelonious Monk", instrument: "Piano", era: "1940s-70s",
        listen_for: "Angular lines, space, and \"wrong\" notes that are exactly right; rhythm as melody.",
        born: "1917, Rocky Mount, North Carolina", died: "1982",
        bio: "The high priest of bebop, and its strangest architect. Monk built a complete \
              musical world - crushed seconds, whole-tone runs, silence used like a fist - \
              and waited, mostly in obscurity, for everyone else to catch up.",
        life: &[
            ("1917", "Born in Rocky Mount, North Carolina; the family moves to San Juan Hill, Manhattan, when he is five."),
            ("1941", "House pianist at Minton's Playhouse, where the after-hours experiments become bebop."),
            ("1947", "First Blue Note sessions as a leader - \"'Round Midnight\" among them - to almost no commercial notice."),
            ("1951", "Takes the fall when drugs are found in a car he shares with Bud Powell; New York revokes his cabaret card and he cannot play the city's clubs for six years."),
            ("1957", "The card restored, his residency at the Five Spot with John Coltrane becomes the stuff of legend."),
            ("1964", "On the cover of Time - one of the very few jazz musicians ever to appear there."),
            ("1970s", "Withdraws from public life, living quietly at the home of his patron, the Baroness Nica de Koenigswarter."),
            ("1982", "Dies in Weehawken, New Jersey; his son and the repertory movement carry the music on."),
        ] },
    Master { name: "Charles Mingus", instrument: "Bass / composer", era: "1940s-70s",
        listen_for: "Gospel-and-blues fire, collective improvisation, huge dynamics.",
        born: "1922, Nogales, Arizona", died: "1979",
        bio: "Jazz's volcanic composer. Mingus fused the Black church, Ellington, and the \
              avant-garde into music that shouts, whispers and riots - often inside one chorus - \
              and fought, loudly and in public, for the dignity of the people who made it.",
        life: &[
            ("1922", "Born in Nogales, Arizona and raised in Watts; the music of the Holiness church never leaves his ears."),
            ("1947", "Records \"Mingus Fingers\" with Lionel Hampton - a bassist who writes, which was not supposed to happen."),
            ("1952", "Founds Debut Records with Max Roach so musicians can own their own work."),
            ("1953", "Plays - and releases - the Massey Hall concert with Parker, Gillespie, Powell and Roach: bebop's last summit."),
            ("1956", "\"Pithecanthropus Erectus\": composition by dictation and ear, the Jazz Workshop method arrives."),
            ("1959", "\"Mingus Ah Um\" - the elegy \"Goodbye Pork Pie Hat\" beside the protest of \"Fables of Faubus\", in one album."),
            ("1962", "The Town Hall concert collapses in under-rehearsed chaos - his grandest ambition, publicly wrecked."),
            ("1971", "Publishes his wild memoir, \"Beneath the Underdog\"."),
            ("1977", "Diagnosed with ALS; composes to the end, dictating when he can no longer play."),
            ("1979", "Dies in Cuernavaca, Mexico; his ashes are scattered in the Ganges."),
        ] },
    Master { name: "Miles Davis", instrument: "Trumpet", era: "1940s-90s",
        listen_for: "Space and reinvention - few notes, deep lyricism; pioneered modal jazz.",
        born: "1926, Alton, Illinois", died: "1991",
        bio: "Jazz's great reinventor. Five times he heard where the music was about to go and \
              got there first - cool, hard bop, modal, the second quintet, fusion - saying more \
              with silence and a Harmon mute than most players say with everything they have.",
        life: &[
            ("1926", "Born to a prosperous dental surgeon in Alton, Illinois; grows up in East St. Louis."),
            ("1944", "Enrols at Juilliard - mostly, by his own account, to find Charlie Parker; drops out and joins him."),
            ("1949", "The \"Birth of the Cool\" nonet sessions sketch a new, arranged coolness."),
            ("1954", "Locks himself in at his father's farm and kicks heroin alone."),
            ("1955", "A muted \"'Round Midnight\" at Newport relaunches him; the first great quintet, with Coltrane, follows."),
            ("1959", "\"Kind of Blue\": modal jazz, first take, the best-selling jazz record ever made."),
            ("1959", "Beaten and arrested by police outside Birdland between sets of his own gig - the scars, he said, stayed."),
            ("1969", "\"In a Silent Way\" and then \"Bitches Brew\" plug jazz in; fusion begins."),
            ("1975", "Vanishes into a five-year silence of illness and excess."),
            ("1981", "Returns, touring and painting, restless to the end."),
            ("1991", "Dies in Santa Monica, weeks after revisiting the Gil Evans scores at Montreux."),
        ] },
    Master { name: "John Coltrane", instrument: "Tenor / soprano sax", era: "1950s-60s",
        listen_for: "\"Sheets of sound,\" modal jazz, the Giant Steps matrix, spiritual intensity.",
        born: "1926, Hamlet, North Carolina", died: "1967",
        bio: "The great seeker. In one decade Coltrane went from journeyman sideman to the most \
              influential saxophonist alive, treating practice as devotion and the horn as a \
              way to say thank you - harder, higher and freer until the end.",
        life: &[
            ("1926", "Born in Hamlet, North Carolina; his father, grandfather and grandmother all die within months when he is twelve."),
            ("1945", "Navy service in Hawaii; his first recordings are made with a sailors' band."),
            ("1955", "Miles Davis hires him - raw, searching, and about to become unrecognisable."),
            ("1957", "Fired by Miles over heroin; quits cold at home in Philadelphia and describes a spiritual awakening he never walks back."),
            ("1957", "Six months with Monk at the Five Spot - \"I learned from him in every way\"."),
            ("1959", "Records \"Giant Steps\", the third-relation obstacle course that still tests every improviser."),
            ("1960", "Forms the classic quartet with McCoy Tyner, Elvin Jones and Jimmy Garrison; the soprano sax joins on \"My Favorite Things\"."),
            ("1964", "Records \"A Love Supreme\" in a single December evening - a psalm of gratitude in four parts."),
            ("1965", "Pushes into free jazz with \"Ascension\"; audiences split, he does not look back."),
            ("1967", "Dies of liver cancer at 40; a San Francisco congregation later canonises him."),
        ] },
    Master { name: "Stan Getz", instrument: "Tenor sax", era: "1940s-90s",
        listen_for: "\"The Sound\" - a gorgeous tone and effortless, lyrical phrasing; bossa nova.",
        born: "1927, Philadelphia", died: "1991",
        bio: "They called him The Sound. Getz owned the most beautiful tenor tone of his era \
              and twice caught lightning: the cool-toned ballad style that made him famous at \
              21, and the bossa nova records that carried jazz onto the world's pop charts.",
        life: &[
            ("1927", "Born in Philadelphia and raised in the Bronx; a professional by his mid-teens."),
            ("1943", "Joins Jack Teagarden's band at sixteen, with the trombonist as his legal guardian on the road."),
            ("1948", "His eight bars on Woody Herman's \"Early Autumn\" make him famous - The Sound, fully formed at 21."),
            ("1954", "Addiction drags him to an attempted pharmacy hold-up and jail - the low point he spends years climbing out of."),
            ("1958", "Moves to Copenhagen, rebuilding his life and playing across Europe."),
            ("1961", "Records \"Focus\", Eddie Sauter's string suite - the record he was proudest of."),
            ("1962", "\"Jazz Samba\" with Charlie Byrd ignites the bossa nova wave."),
            ("1964", "\"The Girl from Ipanema\", with Joao and Astrud Gilberto; \"Getz/Gilberto\" becomes the first jazz album to win the Grammy for Album of the Year."),
            ("1972", "Champions the young Chick Corea and Stanley Clarke in his own band."),
            ("1991", "Dies in Malibu, sober and still playing beautifully; his last records rank with his first."),
        ] },
    Master { name: "Doc Severinsen", instrument: "Trumpet", era: "1950s-2010s (big band)",
        listen_for: "Big-band virtuosity, brilliant tone, dazzling range and command.",
        born: "1927, Arlington, Oregon", died: "",
        bio: "America's bandleader. For three decades of late nights, Severinsen's flashing \
              trumpet and louder wardrobe led the tightest big band on television - and behind \
              the showman stood one of the most complete trumpet players who ever lived.",
        life: &[
            ("1927", "Born in Arlington, Oregon; \"Little Doc\", the dentist's son, is a prodigy on cornet by seven."),
            ("1940s", "On the road with the Ted Fio Rito, Charlie Barnet and Tommy Dorsey bands while still a teenager."),
            ("1949", "Joins NBC as a staff musician in New York - the start of a forty-year broadcasting life."),
            ("1962", "Joins the Tonight Show band as Johnny Carson arrives."),
            ("1967", "Takes over as the Tonight Show's bandleader; the band, and the wardrobe, become an institution."),
            ("1987", "The Tonight Show Band's album wins the Grammy for best large-ensemble jazz."),
            ("1992", "Steps down with Carson after 25 years, and goes straight back on the road."),
            ("2010s", "Still touring and guest-conducting symphony pops into his nineties."),
        ] },
    Master { name: "Maynard Ferguson", instrument: "Trumpet", era: "1940s-2000s (big band)",
        listen_for: "Stratospheric high-note power and screaming lead lines.",
        born: "1928, Verdun, Quebec", died: "2006",
        bio: "The high-note king. Ferguson played the trumpet's impossible register as if it \
              were the staff, led big bands for half a century, and pulled a generation of \
              young players into jazz through gymnasium concerts and a Rocky theme.",
        life: &[
            ("1928", "Born in Verdun, Quebec; a soloist with the Canadian Broadcasting Corporation while a schoolboy."),
            ("1950", "Joins Stan Kenton's orchestra; his lead work above high C makes him famous overnight."),
            ("1956", "Leads the Birdland Dream Band in New York, then his own hard-driving big band."),
            ("1967", "Walks away at his peak - to England, and to India, teaching and searching."),
            ("1970", "Returns with a British band and a new mission: playing the schools."),
            ("1977", "His \"Gonna Fly Now\" from Rocky goes gold and cracks the pop Top 30 - kids flood the band rooms."),
            ("1990s", "Big Bop Nouveau keeps him on the road two hundred nights a year."),
            ("2006", "Dies in Ventura, California, still booked months ahead."),
        ] },
    Master { name: "Chick Corea", instrument: "Piano", era: "1960s-2020s (fusion / Latin / post-bop)",
        listen_for: "Crystalline lines, Spanish colours, endlessly inventive rhythm.",
        born: "1941, Chelsea, Massachusetts", died: "2021",
        bio: "Jazz piano's brightest colourist. From Miles's electric revolution to Return to \
              Forever's Spanish-tinged fusion to fifty years of duets, Corea composed standards \
              (\"Spain\", \"Windows\", \"500 Miles High\") and never stopped sounding delighted.",
        life: &[
            ("1941", "Born in Chelsea, Massachusetts; his trumpeter father starts him on piano at four."),
            ("1962", "Apprentices in the Latin bands of Mongo Santamaria and Willie Bobo - the clave never leaves his playing."),
            ("1968", "Replaces Herbie Hancock with Miles Davis; his Fender Rhodes helps drive \"Bitches Brew\"."),
            ("1972", "Founds Return to Forever; \"Spain\" becomes one of the most-played jazz compositions on earth."),
            ("1976", "\"Romantic Warrior\" takes fusion to gold-record scale."),
            ("1978", "Begins the duet partnerships - Gary Burton, Herbie Hancock - that run the rest of his life."),
            ("2010s", "Tours constantly with new trios, banks 23 Grammy awards, and teaches everywhere he lands."),
            ("2021", "Dies at 79, leaving a note urging everyone with an inkling to play to do so."),
        ] },
    Master { name: "Arturo Sandoval", instrument: "Trumpet", era: "1970s-2020s (Afro-Cuban)",
        listen_for: "Virtuosity married to bebop and Latin fire; astonishing range.",
        born: "1949, Artemisa, Cuba", died: "",
        bio: "Cuba's trumpet virtuoso, and living proof of what Gillespie started. Sandoval \
              plays bebop, classical and Afro-Cuban music with the same ferocious command - \
              and paid for his freedom to play it with exile from home.",
        life: &[
            ("1949", "Born in Artemisa, Cuba; teaches himself on a borrowed trumpet in the village band."),
            ("1973", "Co-founds Irakere with Chucho Valdes and Paquito D'Rivera - Cuban jazz's great engine."),
            ("1977", "Volunteers as Dizzy Gillespie's driver in Havana without admitting he plays; Gillespie finds out on the bandstand and becomes his mentor."),
            ("1979", "Irakere's debut album wins Cuba its first Grammy."),
            ("1990", "Defects to the American embassy in Rome while touring with Gillespie's United Nation Orchestra, his family beside him."),
            ("1998", "Becomes a United States citizen after a long fight."),
            ("2000", "His story reaches film in HBO's \"For Love or Country\", with Andy Garcia."),
            ("2013", "Receives the Presidential Medal of Freedom."),
            ("2020s", "Still touring, recording and teaching - range and fire undimmed."),
        ] },
];

/// Further listening, named rather than described.
pub const ALSO_EXPLORE: &str =
    "Ella Fitzgerald, Duke Ellington, Count Basie, Sarah Vaughan, Bill Evans, Sonny Rollins, \
     Wes Montgomery, Herbie Hancock, Clifford Brown, Freddie Hubbard, Oscar Peterson, \
     Wynton Marsalis.";

/// A tune worth knowing, and why.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Standard {
    pub title: &'static str,
    pub composer: &'static str,
    pub key_and_form: &'static str,
    pub teaches: &'static str,
    /// Id of the matching progression in [`crate::progression::all`], when the
    /// changes are playable here.
    pub progression_id: Option<&'static str>,
    /// An idiomatic jam tempo in bpm — how the tune is actually played, so a
    /// one-click jam starts at the right speed instead of whatever the tempo
    /// slider was left on.
    pub tempo: f32,
    /// Whether the tune swings; `false` means straight eighths (Latin feel).
    pub swing: bool,
}

pub const STANDARDS: &[Standard] = &[
    Standard { title: "So What", composer: "Miles Davis",
        key_and_form: "D Dorian - 32-bar AABA",
        teaches: "Modal soloing and the power of space.",
        progression_id: Some("sowhat"), tempo: 136.0, swing: true },
    Standard { title: "Blue Bossa", composer: "Kenny Dorham",
        key_and_form: "C minor - 16 bars",
        teaches: "Minor ii-V-i, a key change, and Latin feel.",
        progression_id: Some("bluebossa"), tempo: 150.0, swing: false },
    Standard { title: "Take the A Train", composer: "Billy Strayhorn",
        key_and_form: "C major - 32-bar AABA",
        teaches: "Ellington's theme, written by Strayhorn; the famous D7(#11).",
        progression_id: Some("atrain"), tempo: 160.0, swing: true },
    Standard { title: "Rhythm Changes (from \"I Got Rhythm\")", composer: "George Gershwin",
        key_and_form: "played in Bb - 32-bar AABA",
        teaches: "Fast ii-Vs plus a dominant-cycle bridge - the bebop proving ground.",
        progression_id: Some("rhythmfull"), tempo: 220.0, swing: true },
    Standard { title: "Autumn Leaves", composer: "Joseph Kosma",
        key_and_form: "G minor / Bb - 32 bars",
        teaches: "Major and minor ii-V-I side by side.",
        progression_id: Some("autumn"), tempo: 120.0, swing: true },
    Standard { title: "Blues (Now's the Time)", composer: "Charlie Parker",
        key_and_form: "F - 12-bar blues",
        teaches: "The foundation - feel plus chord tones.",
        progression_id: Some("blues"), tempo: 145.0, swing: true },
    Standard { title: "Giant Steps", composer: "John Coltrane",
        key_and_form: "B major - 16 bars",
        teaches: "The major-third matrix; the ultimate changes-running challenge. (Plays here as the same cycle in C.)",
        progression_id: Some("coltrane"), tempo: 230.0, swing: true },
    Standard { title: "All the Things You Are", composer: "Jerome Kern",
        key_and_form: "Ab major - 36 bars",
        teaches: "Chord roots cycling in fourths through shifting key centres.",
        progression_id: Some("attya"), tempo: 130.0, swing: true },
    Standard { title: "All of Me", composer: "Gerald Marks & Seymour Simons",
        key_and_form: "C major - 32-bar ABAC",
        teaches: "Secondary dominants in a row - each V7 pointing at the next stop.",
        progression_id: Some("allofme"), tempo: 140.0, swing: true },
    Standard { title: "Fly Me to the Moon", composer: "Bart Howard",
        key_and_form: "C major - 32 bars",
        teaches: "The circle of fourths sung: vi-ii-V-I rolling forward.",
        progression_id: Some("flymoon"), tempo: 120.0, swing: true },
    Standard { title: "Misty", composer: "Erroll Garner",
        key_and_form: "Eb major - 32-bar AABA",
        teaches: "Ballad playing: ii-Vs at a slow tempo, and the backdoor Db7 home.",
        progression_id: Some("misty"), tempo: 72.0, swing: true },
    Standard { title: "Tune Up", composer: "Eddie \"Cleanhead\" Vinson",
        key_and_form: "D major - 16 bars",
        teaches: "Three ii-V-Is a whole step apart; long issued under Miles Davis's name.",
        progression_id: Some("tuneup"), tempo: 200.0, swing: true },
    Standard { title: "Solar", composer: "Chuck Wayne",
        key_and_form: "C minor - 12 bars",
        teaches: "Key centres falling by fourths out of a minor home; also issued as Miles Davis's.",
        progression_id: Some("solar"), tempo: 160.0, swing: true },
    Standard { title: "Cantaloupe Island", composer: "Herbie Hancock",
        key_and_form: "F minor - 16-bar vamp",
        teaches: "Groove over three chords - rhythm is the solo's engine.",
        progression_id: Some("cantaloupe"), tempo: 112.0, swing: false },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progression;

    #[test]
    fn every_master_is_fully_described() {
        for m in MASTERS {
            assert!(!m.name.is_empty() && !m.instrument.is_empty());
            assert!(!m.era.is_empty(), "{} has no era", m.name);
            assert!(
                m.listen_for.len() > 30,
                "{} needs a real listening note, not a label",
                m.name
            );
        }
    }

    #[test]
    fn every_era_gives_a_decade_range() {
        // A bare genre ("big band") tells a student nothing about when to look.
        for m in MASTERS {
            assert!(
                m.era.contains("s-"),
                "{} has era '{}' with no decade range",
                m.name,
                m.era
            );
        }
    }

    fn year_of(text: &str) -> Option<i32> {
        let digits: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
        (digits.len() == 4).then(|| digits.parse().unwrap())
    }

    /// A biography page needs a real story: an introduction, a birthplace, and
    /// a timeline that moves forward through a plausible span of years.
    #[test]
    fn every_master_has_a_life_worth_reading() {
        for m in MASTERS {
            assert!(m.bio.len() > 100, "{}'s bio is a stub", m.name);
            assert!(!m.born.is_empty(), "{} has no birth entry", m.name);
            assert!(m.life.len() >= 5, "{}'s timeline is too thin", m.name);
            let mut last = 0;
            for (year, event) in m.life {
                let y = year_of(year)
                    .unwrap_or_else(|| panic!("{}: '{year}' does not start with a year", m.name));
                assert!((1900..=2030).contains(&y), "{}: implausible year {y}", m.name);
                assert!(y >= last, "{}: timeline goes backwards at {year}", m.name);
                assert!(event.len() > 20, "{}: event at {year} says nothing", m.name);
                last = y;
            }
        }
    }

    /// Two of these masters are alive. Declaring a living person dead is the
    /// kind of "fact" this file must never invent.
    #[test]
    fn the_living_are_not_declared_dead() {
        let by = |n: &str| MASTERS.iter().find(|m| m.name == n).unwrap();
        assert!(by("Doc Severinsen").died.is_empty());
        assert!(by("Arturo Sandoval").died.is_empty());
        // And the departed all have a plausible year that follows their birth.
        for m in MASTERS {
            if !m.died.is_empty() {
                let born = year_of(m.born).unwrap();
                let died = year_of(m.died).unwrap();
                assert!(died > born && died <= 2026, "{}: birth {born}, death {died}", m.name);
            }
        }
    }

    /// Strayhorn wrote it. Crediting Ellington repeats exactly the erasure he is
    /// famous for suffering.
    #[test]
    fn take_the_a_train_is_credited_to_strayhorn() {
        let s = STANDARDS.iter().find(|s| s.title.contains("A Train")).unwrap();
        assert_eq!(s.composer, "Billy Strayhorn");
        assert!(!s.composer.contains("Ellington"));
    }

    #[test]
    fn rhythm_changes_distinguishes_the_jazz_key_from_the_original() {
        let s = STANDARDS.iter().find(|s| s.title.starts_with("Rhythm Changes")).unwrap();
        assert_eq!(s.composer, "George Gershwin");
        // Bb / 32 bars is the jazz vehicle; the song itself is Db and 34 bars.
        assert!(s.key_and_form.contains("Bb"));
        assert!(s.key_and_form.contains("played in"));
    }

    #[test]
    fn composer_credits_are_the_verified_ones() {
        let by = |t: &str| STANDARDS.iter().find(|s| s.title.starts_with(t)).unwrap();
        assert_eq!(by("So What").composer, "Miles Davis");
        assert_eq!(by("Blue Bossa").composer, "Kenny Dorham");
        assert_eq!(by("Autumn Leaves").composer, "Joseph Kosma");
        assert_eq!(by("Giant Steps").composer, "John Coltrane");
        assert_eq!(by("All the Things").composer, "Jerome Kern");
        assert_eq!(by("All of Me").composer, "Gerald Marks & Seymour Simons");
        assert_eq!(by("Fly Me to the Moon").composer, "Bart Howard");
        assert_eq!(by("Misty").composer, "Erroll Garner");
        assert_eq!(by("Cantaloupe Island").composer, "Herbie Hancock");
    }

    /// Like the A Train: "Tune Up" and "Solar" were issued under Miles Davis's
    /// name, but the evidence credits Vinson and Wayne. The teaching note keeps
    /// the Davis connection; the composer field keeps the writer.
    #[test]
    fn tunes_issued_under_miless_name_credit_their_writers() {
        let by = |t: &str| STANDARDS.iter().find(|s| s.title.starts_with(t)).unwrap();
        assert_eq!(by("Tune Up").composer, "Eddie \"Cleanhead\" Vinson");
        assert!(by("Tune Up").teaches.contains("Miles Davis"));
        assert_eq!(by("Solar").composer, "Chuck Wayne");
        assert!(by("Solar").teaches.contains("Miles Davis"));
    }

    /// The jam tempos are how the tunes are actually played, inside the range
    /// the transport can play. The two easy to get wrong: a bossa does not
    /// swing, and rhythm changes without real tempo teaches the wrong lesson.
    #[test]
    fn jam_tempos_are_playable_and_idiomatic() {
        for s in STANDARDS {
            assert!(
                (60.0..=280.0).contains(&s.tempo),
                "{} has jam tempo {} outside the playable range",
                s.title,
                s.tempo
            );
        }
        let bossa = STANDARDS.iter().find(|s| s.title == "Blue Bossa").unwrap();
        assert!(!bossa.swing, "a bossa plays straight eighths, not swung");
        let rc = STANDARDS.iter().find(|s| s.title.starts_with("Rhythm Changes")).unwrap();
        assert!(rc.swing && rc.tempo >= 180.0, "rhythm changes is the fast-swing proving ground");
    }

    /// Every playable standard must point at a progression that really exists,
    /// or the button does nothing.
    #[test]
    fn playable_standards_resolve_to_a_real_progression() {
        let progs = progression::all();
        for s in STANDARDS {
            if let Some(id) = s.progression_id {
                assert!(
                    progs.iter().any(|p| p.id == id),
                    "{} points at unknown progression '{}'",
                    s.title,
                    id
                );
            }
        }
    }

    /// All the Things You Are spent a while listed "study by ear" with no
    /// chart; now that its 36 bars are in the progression catalogue, every
    /// standard is playable — and this keeps it that way.
    #[test]
    fn every_standard_is_playable() {
        for s in STANDARDS {
            assert!(
                s.progression_id.is_some(),
                "{} has no changes to load",
                s.title
            );
        }
    }

    #[test]
    fn titles_are_unique() {
        let mut t: Vec<&str> = STANDARDS.iter().map(|s| s.title).collect();
        t.sort_unstable();
        let n = t.len();
        t.dedup();
        assert_eq!(n, t.len(), "duplicate standard");
        let mut m: Vec<&str> = MASTERS.iter().map(|x| x.name).collect();
        m.sort_unstable();
        let n = m.len();
        m.dedup();
        assert_eq!(n, m.len(), "duplicate master");
    }
}

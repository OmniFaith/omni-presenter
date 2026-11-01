//! A shared offline catalogue for the native picker, reactions and web clients.
#[derive(Clone, serde::Serialize)]
pub struct Emoji {
    pub value: &'static str,
    pub name: &'static str,
    #[serde(skip)]
    pub normalized: String,
    #[serde(skip)]
    pub category: usize,
}
#[derive(serde::Serialize)]
pub struct Category {
    pub name: &'static str,
    pub icon: &'static str,
    pub emojis: Vec<Emoji>,
}

pub fn catalogue() -> &'static [Category] {
    static CATALOGUE: std::sync::OnceLock<Vec<Category>> = std::sync::OnceLock::new();
    CATALOGUE.get_or_init(build_catalogue)
}

fn build_catalogue() -> Vec<Category> {
    [
        (
            "Rostos",
            "😀",
            vec![
                ("😀", "feliz sorriso"),
                ("😃", "alegre"),
                ("😄", "rindo"),
                ("😁", "sorriso"),
                ("😆", "risada"),
                ("😅", "suor alívio"),
                ("😂", "risos lágrimas"),
                ("🤣", "gargalhada"),
                ("😊", "sorridente"),
                ("🙂", "sorriso suave"),
                ("😉", "piscada"),
                ("😍", "apaixonado"),
                ("🥰", "carinho"),
                ("😘", "beijo"),
                ("😎", "óculos legal"),
                ("🤩", "estrelas"),
                ("🥳", "festa"),
                ("😇", "anjo"),
                ("🤗", "abraço"),
                ("🤔", "pensando"),
                ("🤫", "silêncio"),
                ("🤭", "surpresa"),
                ("😐", "neutro"),
                ("🙄", "olhos"),
                ("😴", "sono"),
                ("😢", "triste"),
                ("😭", "chorando"),
                ("😮", "surpreso"),
                ("😱", "espanto"),
                ("😬", "nervoso"),
                ("😌", "alívio"),
                ("🥺", "por favor"),
                ("🤓", "estudioso"),
                ("😋", "gostoso"),
                ("😜", "brincadeira"),
                ("🫡", "saudação"),
                ("😏", "sorriso maroto"),
                ("😒", "desdém"),
                ("😞", "decepção"),
                ("😔", "pensativo"),
                ("😟", "preocupado"),
                ("😕", "confuso"),
                ("😣", "esforço"),
                ("😖", "frustrado"),
                ("😫", "cansado"),
                ("😩", "exausto"),
                ("😤", "bufando"),
                ("😠", "irritado"),
                ("😡", "furioso"),
                ("🤬", "bravo"),
                ("😳", "envergonhado"),
                ("🥵", "calor"),
                ("🥶", "frio"),
                ("😨", "medo"),
                ("😰", "ansioso"),
                ("😥", "aliviado"),
                ("😓", "suor"),
                ("🤯", "mente explodindo"),
                ("🥱", "bocejo"),
                ("🤤", "babando"),
                ("😪", "sonolento"),
                ("🤐", "boca fechada"),
                ("🤑", "dinheiro"),
                ("🤠", "cowboy"),
                ("🥴", "tonto"),
                ("😵", "atordoado"),
                ("🤢", "enjoo"),
                ("🤮", "vômito"),
                ("🤧", "espirro"),
                ("😷", "máscara"),
                ("🤒", "doente"),
                ("🤕", "machucado"),
            ],
        ),
        (
            "Gestos",
            "👍",
            vec![
                ("👍", "positivo sim"),
                ("👎", "negativo não"),
                ("👏", "palmas"),
                ("🙌", "celebrar louvar"),
                ("🙏", "oração obrigado"),
                ("🤝", "acordo"),
                ("💪", "força"),
                ("👌", "certo"),
                ("🤌", "gesto"),
                ("✌️", "paz"),
                ("🤞", "esperança"),
                ("🤟", "amor"),
                ("🤘", "música"),
                ("👋", "olá tchau"),
                ("🤚", "pare"),
                ("✋", "mão"),
                ("🖐️", "cinco"),
                ("👊", "unidos"),
                ("✊", "luta"),
                ("☝️", "atenção"),
                ("👆", "acima"),
                ("👇", "abaixo"),
                ("👈", "esquerda"),
                ("👉", "direita"),
                ("🫶", "coração mãos"),
                ("👐", "mãos abertas"),
                ("💅", "unhas"),
                ("👀", "olhando"),
                ("👂", "ouvir"),
                ("🧠", "ideia cérebro"),
                ("🤲", "mãos juntas"),
                ("🙇", "curvando"),
                ("🙋", "mão levantada"),
                ("🙆", "ok"),
                ("🙅", "não"),
                ("💁", "informação"),
                ("🙎", "emburrado"),
                ("🤦", "facepalm"),
                ("🤷", "ombros"),
                ("🖖", "saudação vulcana"),
                ("🤙", "hang loose"),
                ("✍️", "escrevendo"),
                ("🤛", "punho esquerdo"),
                ("🤜", "punho direito"),
                ("🦾", "braço robô"),
                ("🦿", "perna robô"),
                ("🦵", "perna"),
                ("🦶", "pé"),
                ("👃", "nariz"),
                ("👄", "boca"),
                ("👅", "língua"),
                ("🦷", "dente"),
                ("🦴", "osso"),
            ],
        ),
        (
            "Corações e fé",
            "❤️",
            vec![
                ("❤️", "amor coração vermelho"),
                ("🧡", "laranja"),
                ("💛", "amarelo"),
                ("💚", "verde"),
                ("💙", "azul"),
                ("💜", "roxo"),
                ("🖤", "preto"),
                ("🤍", "branco"),
                ("🤎", "marrom"),
                ("💖", "brilho amor"),
                ("💗", "coração crescendo"),
                ("💕", "dois corações"),
                ("💞", "corações"),
                ("💓", "batida"),
                ("💘", "flecha"),
                ("💝", "presente"),
                ("💟", "coração"),
                ("❣️", "carinho"),
                ("💔", "coração partido"),
                ("✝️", "cruz fé"),
                ("⛪", "igreja"),
                ("🕊️", "pomba paz"),
                ("📖", "bíblia livro"),
                ("🛐", "adoração oração"),
                ("✨", "brilho"),
                ("🌟", "estrela"),
                ("💌", "carta de amor"),
                ("💐", "buquê"),
                ("☦️", "cruz ortodoxa"),
                ("☪️", "islã"),
                ("🕉️", "om"),
                ("✡️", "estrela de davi"),
                ("☸️", "roda do dharma"),
                ("📿", "rosário"),
                ("🕯️", "vela"),
                ("👼", "anjo"),
                ("🕌", "mesquita"),
                ("🕍", "sinagoga"),
                ("🛕", "templo hindu"),
                ("🕋", "caaba"),
                ("💒", "capela"),
                ("🧿", "olho grego"),
                ("🔯", "estrela de seis pontas"),
                ("🕎", "menorá"),
                ("☯️", "yin yang"),
                ("🔱", "tridente"),
                ("⚛️", "átomo"),
            ],
        ),
        (
            "Música e equipe",
            "🎵",
            vec![
                ("🎵", "nota musical"),
                ("🎶", "música notas"),
                ("🎤", "microfone"),
                ("🎧", "fone"),
                ("🎼", "partitura"),
                ("🎹", "piano teclado"),
                ("🎸", "guitarra violão"),
                ("🥁", "bateria"),
                ("🎺", "trompete"),
                ("🎷", "saxofone"),
                ("🎻", "violino"),
                ("🪕", "banjo"),
                ("🔊", "alto falante"),
                ("🔉", "volume"),
                ("🔇", "mudo"),
                ("🎬", "claquete vídeo"),
                ("📷", "câmera foto"),
                ("📹", "filmadora"),
                ("📺", "tela televisão"),
                ("💻", "computador"),
                ("📱", "celular"),
                ("🎛️", "mesa de som"),
                ("🎚️", "controle volume"),
                ("🔌", "energia cabo"),
                ("💡", "ideia luz"),
                ("📡", "rede"),
                ("📢", "aviso"),
                ("🔔", "sino notificação"),
                ("📋", "escala lista"),
                ("📝", "anotação"),
                ("🎙️", "microfone estúdio"),
                ("🪘", "tambor"),
                ("🪗", "acordeão"),
                ("📻", "rádio"),
                ("📣", "megafone"),
                ("📽️", "projetor"),
                ("🎞️", "filme"),
                ("🎥", "câmera de vídeo"),
                ("📸", "câmera fotográfica"),
                ("🖥️", "computador"),
                ("⌨️", "teclado"),
                ("🖱️", "mouse"),
                ("🖨️", "impressora"),
                ("🎮", "controle"),
                ("🕹️", "joystick"),
                ("📀", "cd"),
                ("💿", "dvd"),
                ("📼", "fita"),
                ("💾", "disquete"),
                ("📞", "telefone"),
                ("☎️", "telefone fixo"),
                ("📟", "pager"),
                ("📠", "fax"),
            ],
        ),
        (
            "Celebração e natureza",
            "🎉",
            vec![
                ("🎉", "celebração festa"),
                ("🎊", "confete"),
                ("🎈", "balão"),
                ("🎁", "presente"),
                ("🎂", "aniversário bolo"),
                ("🏆", "troféu"),
                ("🥇", "medalha"),
                ("🔥", "fogo"),
                ("💥", "explosão"),
                ("💯", "cem perfeito"),
                ("⭐", "estrela"),
                ("🌈", "arco íris"),
                ("☀️", "sol"),
                ("🌤️", "sol nuvens"),
                ("🌙", "lua"),
                ("☁️", "nuvem"),
                ("⚡", "raio"),
                ("🌧️", "chuva"),
                ("🌊", "ondas"),
                ("🌻", "girassol"),
                ("🌹", "rosa"),
                ("🌺", "flor"),
                ("🌷", "tulipa"),
                ("🌿", "folhas"),
                ("🍀", "sorte"),
                ("🌱", "crescer planta"),
                ("🦋", "borboleta"),
                ("🐦", "pássaro"),
                ("🦁", "leão"),
                ("🐑", "ovelha"),
                ("🎇", "fogos"),
                ("🧨", "bombinha"),
                ("🍾", "champanhe"),
                ("🥂", "brinde"),
                ("🍻", "cerveja"),
                ("🍷", "vinho"),
                ("🍕", "pizza"),
                ("🍞", "pão"),
                ("🍇", "uva"),
                ("🍎", "maçã"),
                ("🍊", "laranja"),
                ("🍋", "limão"),
                ("🍓", "morango"),
                ("🍉", "melancia"),
                ("🥥", "coco"),
                ("🥑", "abacate"),
                ("🌲", "pinheiro"),
                ("🌵", "cacto"),
                ("🌾", "trigo"),
                ("🌸", "flor"),
                ("🍁", "folha de outono"),
                ("❄️", "neve"),
                ("☃️", "boneco de neve"),
                ("🌍", "planeta terra"),
                ("🪐", "saturno"),
                ("🌞", "sol com rosto"),
                ("🌝", "lua cheia"),
                ("🪴", "planta"),
                ("🌼", "margarida"),
                ("🐶", "cachorro"),
                ("🐱", "gato"),
                ("🐟", "peixe"),
                ("🐴", "cavalo"),
            ],
        ),
        (
            "Sinais e objetos",
            "✅",
            vec![
                ("✅", "feito certo"),
                ("❌", "errado cancelar"),
                ("⚠️", "atenção aviso"),
                ("❗", "importante"),
                ("❓", "pergunta"),
                ("💬", "mensagem"),
                ("💭", "pensamento"),
                ("💤", "dormir"),
                ("🔴", "vermelho"),
                ("🟠", "laranja"),
                ("🟡", "amarelo"),
                ("🟢", "verde"),
                ("🔵", "azul"),
                ("🟣", "roxo"),
                ("⚪", "branco"),
                ("⚫", "preto"),
                ("▶️", "play iniciar"),
                ("⏸️", "pausar"),
                ("⏹️", "parar"),
                ("🔁", "repetir refrão"),
                ("🔀", "aleatório"),
                ("⏰", "horário"),
                ("⏳", "aguardar"),
                ("📅", "agenda data"),
                ("📌", "fixar"),
                ("🔗", "link"),
                ("📎", "anexo arquivo"),
                ("📁", "pasta"),
                ("🔑", "chave"),
                ("☕", "café"),
                ("✔️", "confirmado"),
                ("☑️", "caixa marcada"),
                ("❎", "caixa x"),
                ("⁉️", "exclamação interrogação"),
                ("‼️", "dupla exclamação"),
                ("➕", "mais"),
                ("➖", "menos"),
                ("➗", "divisão"),
                ("✖️", "multiplicação"),
                ("🔒", "cadeado"),
                ("🔓", "cadeado aberto"),
                ("🔐", "chave segura"),
                ("🛠️", "ferramentas"),
                ("⚙️", "engrenagem"),
                ("🧰", "caixa de ferramentas"),
                ("🚪", "porta"),
                ("🪑", "cadeira"),
                ("🧹", "vassoura"),
                ("🗑️", "lixeira"),
                ("📦", "caixa"),
                ("📍", "local"),
                ("🔖", "marcador"),
                ("🗂️", "pastas"),
                ("📂", "pasta aberta"),
                ("🖇️", "clipes"),
                ("📏", "régua"),
                ("✂️", "tesoura"),
                ("🖊️", "caneta"),
                ("✏️", "lápis"),
                ("🖌️", "pincel"),
                ("🧽", "esponja"),
                ("🚿", "chuveiro"),
                ("🛒", "carrinho"),
                ("🎯", "alvo"),
                ("🧭", "bússola"),
                ("🗺️", "mapa"),
            ],
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(category, (name, icon, emojis))| Category {
        name,
        icon,
        emojis: emojis
            .into_iter()
            .map(|(value, name)| {
                let labels = annotations().get(value);
                let normalized = crate::core::bible::fold(&format!(
                    "{value} {name} {} {}",
                    labels.map_or("", |labels| labels[0].as_str()),
                    labels.map_or("", |labels| labels[1].as_str())
                ));
                Emoji {
                    value,
                    name,
                    normalized,
                    category,
                }
            })
            .collect(),
    })
    .collect()
}

pub fn all() -> &'static [&'static Emoji] {
    static ALL: std::sync::OnceLock<Vec<&'static Emoji>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        catalogue()
            .iter()
            .flat_map(|category| category.emojis.iter())
            .collect()
    })
}

pub fn valid(value: &str) -> bool {
    static VALID: std::sync::OnceLock<std::collections::HashSet<&'static str>> =
        std::sync::OnceLock::new();
    VALID
        .get_or_init(|| {
            catalogue()
                .iter()
                .flat_map(|category| category.emojis.iter().map(|emoji| emoji.value))
                .collect()
        })
        .contains(value)
}

impl Emoji {
    pub fn label(&self, language: crate::core::i18n::Language) -> &str {
        match language {
            crate::core::i18n::Language::Portuguese => self.name,
            language => annotations()
                .get(self.value)
                .map(|labels| {
                    labels[usize::from(language == crate::core::i18n::Language::Spanish)].as_str()
                })
                .unwrap_or(self.name),
        }
    }
}

impl Category {
    pub fn label(&self, language: crate::core::i18n::Language) -> &str {
        language.text(match self.name {
            "Rostos" => "emoji-faces",
            "Gestos" => "emoji-gestures",
            "Corações e fé" => "emoji-hearts",
            "Música e equipe" => "emoji-music",
            "Celebração e natureza" => "emoji-nature",
            _ => "emoji-objects",
        })
    }
}

fn annotations() -> &'static std::collections::BTreeMap<String, [String; 2]> {
    static ANNOTATIONS: std::sync::OnceLock<std::collections::BTreeMap<String, [String; 2]>> =
        std::sync::OnceLock::new();
    ANNOTATIONS.get_or_init(|| {
        serde_json::from_str(include_str!("i18n/emoji.json")).expect("invalid emoji annotations")
    })
}
pub const QUICK_REACTIONS: [&str; 8] = ["👍", "❤️", "😂", "🙏", "🙌", "🎉", "👀", "🔥"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_match_portuguese_english_and_spanish_without_rebuilding_the_catalogue() {
        let mut cache = crate::core::search::SearchCache::default();
        let all = all();
        for query in ["oração", "prayer", "oración"] {
            assert!(
                cache
                    .filter(query, 0, all.iter().map(|emoji| emoji.normalized.as_str()))
                    .iter()
                    .any(|index| all[*index].value == "🙏")
            );
        }
        let emoji = all.iter().find(|emoji| emoji.value == "🙏").unwrap();
        assert_eq!(
            emoji.label(crate::core::i18n::Language::English),
            "prayer thank you"
        );
        assert_eq!(
            emoji.label(crate::core::i18n::Language::Spanish),
            "oración gracias"
        );
    }

    #[test]
    fn catalogue_is_large_unique_and_searchable() {
        let catalogue = catalogue();
        let mut seen = std::collections::HashSet::new();
        let mut total = 0;
        for category in catalogue {
            assert!(!category.emojis.is_empty(), "{}", category.name);
            for emoji in &category.emojis {
                assert!(!emoji.value.is_empty());
                assert!(!emoji.name.is_empty());
                assert!(
                    annotations().contains_key(emoji.value),
                    "missing translations: {}",
                    emoji.value
                );
                assert!(seen.insert(emoji.value), "duplicado: {}", emoji.value);
                total += 1;
            }
        }
        assert!(total >= 300, "catálogo pequeno: {total}");
        assert!(valid("🙏"));
        assert!(!valid("texto comum"));
    }
}

//! Ukrainian / English UI chrome. Fonts: system TTF with Cyrillic when present.

use aira_desktop_runtime::UiLang;

/// Install a proportional font that can render Ukrainian, if the OS has one.
pub fn install_cyrillic_font(ctx: &egui::Context) {
    let candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/gnu-free/FreeSans.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
        r"C:\Windows\Fonts\arial.ttf",
        r"C:\Windows\Fonts\segoeui.ttf",
    ];
    for path in candidates {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("cyrillic".to_owned(), egui::FontData::from_owned(bytes));
        if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            fam.insert(0, "cyrillic".to_owned());
        }
        if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            fam.push("cyrillic".to_owned());
        }
        ctx.set_fonts(fonts);
        return;
    }
}

/// Static chrome strings for one language.
pub struct Labels {
    pub window_title: &'static str,
    pub heading: &'static str,
    pub subtitle: &'static str,
    pub tab_work: &'static str,
    pub tab_node: &'static str,
    pub tab_network: &'static str,
    pub tab_settings: &'static str,
    pub status: &'static str,
    pub peer: &'static str,
    pub start: &'static str,
    pub stop: &'static str,
    pub refresh: &'static str,
    pub quit: &'static str,
    pub restart_hint: &'static str,
    pub work_heading: &'static str,
    pub work_hint: &'static str,
    pub work_submit: &'static str,
    pub work_not_llm: &'static str,
    pub work_answer: &'static str,
    pub work_verification: &'static str,
    pub work_ids: &'static str,
    pub work_problem_id: &'static str,
    pub work_artifact_id: &'static str,
    pub work_execution_id: &'static str,
    pub work_field_id: &'static str,
    pub work_details: &'static str,
    pub work_no_answer: &'static str,
    pub network_profile: &'static str,
    pub p0: &'static str,
    pub p1: &'static str,
    pub p2: &'static str,
    pub peer_listen: &'static str,
    pub save_listen: &'static str,
    pub advanced: &'static str,
    pub advanced_hint: &'static str,
    pub p3_relay: &'static str,
    pub p4_gossip: &'static str,
    pub relay_ttl: &'static str,
    pub save_ttl: &'static str,
    pub federation: &'static str,
    pub federation_hint: &'static str,
    pub import_federation: &'static str,
    pub discovery: &'static str,
    pub discovery_hint: &'static str,
    pub stun_server: &'static str,
    pub stun_query: &'static str,
    pub discv_to: &'static str,
    pub discv_addr: &'static str,
    pub discv_announce: &'static str,
    pub find_key: &'static str,
    pub find_to: &'static str,
    pub discv_find: &'static str,
    pub settings_heading: &'static str,
    pub language: &'static str,
    pub lang_uk: &'static str,
    pub lang_en: &'static str,
    pub open_window_on_login: &'static str,
    pub open_window_hint: &'static str,
    pub autostart: &'static str,
    pub not_llm: &'static str,
    pub friend_invite: &'static str,
    pub invite_hint: &'static str,
    pub stop_camera: &'static str,
    pub export_json: &'static str,
    pub import_json: &'static str,
    pub show_qr: &'static str,
    pub export_qr: &'static str,
    pub import_qr: &'static str,
    pub scan_qr: &'static str,
    pub scan_camera: &'static str,
    pub peer_off_p0: &'static str,
    pub st_stopped: &'static str,
    pub st_starting: &'static str,
    pub st_running: &'static str,
    pub st_unhealthy: &'static str,
    pub st_stopping: &'static str,
    pub st_failed: &'static str,
}

impl Labels {
    pub fn get(lang: UiLang) -> &'static Labels {
        match lang {
            UiLang::Uk => &UK,
            UiLang::En => &EN,
        }
    }
}

static EN: Labels = Labels {
    window_title: "AIRA Desktop",
    heading: "AIRA Desktop",
    subtitle: "Developer Preview · Problem Statement → Verified Result Artifact",
    tab_work: "Work",
    tab_node: "Node",
    tab_network: "Network",
    tab_settings: "Settings",
    status: "Status:",
    peer: "Peer:",
    start: "Start",
    stop: "Stop",
    refresh: "Refresh",
    quit: "Quit",
    restart_hint: "Profile/listen changed — Stop then Start to apply peer.",
    work_heading: "Problem",
    work_hint: "Submit text to the local node (`POST /v1/problems`, same path as `aira problem submit`). C1 example: Calculate 2 + 2. Other text uses generate-local.",
    work_submit: "Submit",
    work_not_llm: "C1 `Calculate 2 + 2` uses `execution-basic` / `math.eval.safe` (VERIFIED). Other prompts run `text.generate.local` on the local Execution CSU (MockBackend in CI; status executed, not a Verified Result). Generate fail-closes without Phase D activate — this tab never fakes VERIFIED. Core does not host inference.",
    work_answer: "Answer",
    work_verification: "Verification:",
    work_ids: "Identifiers",
    work_problem_id: "Problem ID:",
    work_artifact_id: "Verified artifact:",
    work_execution_id: "Execution artifact:",
    work_field_id: "Field artifact:",
    work_details: "Details",
    work_no_answer: "(no result payload)",
    network_profile: "Network profile",
    p0: "P0 local HTTP",
    p1: "P1 + peer listen",
    p2: "P2 + DHT book",
    peer_listen: "peer_listen:",
    save_listen: "Save listen",
    advanced: "Advanced",
    advanced_hint: "P3 relay hub (--relay) and P4 gossip (--gossip) are mutually exclusive on one peer listen.",
    p3_relay: "P3 relay hub",
    p4_gossip: "P4 gossip trust",
    relay_ttl: "relay_ttl_days:",
    save_ttl: "Save TTL",
    federation: "Federation (P5)",
    federation_hint: "Local pin: import signed federation descriptor JSON (no remote handshake).",
    import_federation: "Import federation descriptor…",
    discovery: "Discovery (P6 Dev)",
    discovery_hint: "Operator shortcuts only — explicit STUN server; no public STUN default; no auto-trust.",
    stun_server: "stun_server:",
    stun_query: "STUN query",
    discv_to: "discv to:",
    discv_addr: "addr:",
    discv_announce: "discv announce",
    find_key: "find key:",
    find_to: "seed to:",
    discv_find: "discv FIND",
    settings_heading: "Settings",
    language: "Language",
    lang_uk: "Українська",
    lang_en: "English",
    open_window_on_login: "Open window on login autostart",
    open_window_hint: "Only for login autostart. The menu icon and `aira-desktop` always open this window — unchecking this cannot lock you out of Settings.",
    autostart: "Start node on login",
    not_llm: "Core does not host inference. Local models belong in inventory/acquisition CSUs (`aira models scan|list|activate`). That is the intended model layer — not a marketplace and not a ban on using a local LLM as a CSU.",
    friend_invite: "Friend invite",
    invite_hint: "File, QR PNG, or camera scan. Import → trust + address book.",
    stop_camera: "Stop camera scan",
    export_json: "Export JSON…",
    import_json: "Import JSON…",
    show_qr: "Show QR",
    export_qr: "Export QR…",
    import_qr: "Import QR…",
    scan_qr: "Scan QR (camera)",
    scan_camera: "Scanning — point camera at PeerInvite QR…",
    peer_off_p0: "peer off (P0)",
    st_stopped: "stopped",
    st_starting: "starting",
    st_running: "running",
    st_unhealthy: "unhealthy",
    st_stopping: "stopping",
    st_failed: "failed",
};

static UK: Labels = Labels {
    window_title: "AIRA Desktop",
    heading: "AIRA Desktop",
    subtitle: "Developer Preview · Problem Statement → Verified Result Artifact",
    tab_work: "Робота",
    tab_node: "Вузол",
    tab_network: "Мережа",
    tab_settings: "Параметри",
    status: "Стан:",
    peer: "Peer:",
    start: "Старт",
    stop: "Стоп",
    refresh: "Оновити",
    quit: "Вийти",
    restart_hint: "Профіль/listen змінено — Стоп, потім Старт, щоб застосувати peer.",
    work_heading: "Задача",
    work_hint: "Надіслати текст на локальний вузол (`POST /v1/problems`, той самий шлях, що `aira problem submit`). C1 приклад: Calculate 2 + 2. Інший текст — generate-local.",
    work_submit: "Надіслати",
    work_not_llm: "C1 `Calculate 2 + 2` іде через `execution-basic` / `math.eval.safe` (VERIFIED). Інші промпти — `text.generate.local` на локальному Execution CSU (MockBackend у CI; статус executed, не Verified Result). Без Phase D activate generate fail-closed — вкладка не підробляє VERIFIED. Ядро не хостить inference.",
    work_answer: "Відповідь",
    work_verification: "Верифікація:",
    work_ids: "Ідентифікатори",
    work_problem_id: "ID задачі:",
    work_artifact_id: "Верифікований артефакт:",
    work_execution_id: "Артефакт виконання:",
    work_field_id: "Артефакт поля:",
    work_details: "Деталі",
    work_no_answer: "(немає payload результату)",
    network_profile: "Мережевий профіль",
    p0: "P0 лише локальний HTTP",
    p1: "P1 + peer listen",
    p2: "P2 + DHT-книга",
    peer_listen: "peer_listen:",
    save_listen: "Зберегти listen",
    advanced: "Додатково",
    advanced_hint: "P3 relay (--relay) і P4 gossip (--gossip) взаємовиключні на одному peer listen.",
    p3_relay: "P3 relay-хаб",
    p4_gossip: "P4 gossip trust",
    relay_ttl: "relay_ttl_days:",
    save_ttl: "Зберегти TTL",
    federation: "Федерація (P5)",
    federation_hint: "Локальний pin: імпорт підписаного JSON-дескриптора (без віддаленого handshake).",
    import_federation: "Імпорт дескриптора федерації…",
    discovery: "Discovery (P6 Dev)",
    discovery_hint: "Лише операторські скорочення — явний STUN; без публічного STUN за замовчуванням; без auto-trust.",
    stun_server: "stun_server:",
    stun_query: "STUN запит",
    discv_to: "discv to:",
    discv_addr: "addr:",
    discv_announce: "discv announce",
    find_key: "find key:",
    find_to: "seed to:",
    discv_find: "discv FIND",
    settings_heading: "Параметри",
    language: "Мова",
    lang_uk: "Українська",
    lang_en: "English",
    open_window_on_login: "Відкривати вікно при автостарті після логіну",
    open_window_hint: "Лише для автостарту після входу в систему. Іконка меню та `aira-desktop` завжди відкривають це вікно — зняття галочки не блокує доступ до параметрів.",
    autostart: "Запускати вузол після входу в систему",
    not_llm: "Ядро не хостить inference. Локальні моделі — шар inventory/acquisition (`aira models scan|list|activate`). Це канонічний model layer: не маркетплейс і не заборона викликати локальну LLM як CSU.",
    friend_invite: "Запрошення друга",
    invite_hint: "Файл, QR PNG або камера. Імпорт → trust + address book.",
    stop_camera: "Зупинити сканування камери",
    export_json: "Експорт JSON…",
    import_json: "Імпорт JSON…",
    show_qr: "Показати QR",
    export_qr: "Експорт QR…",
    import_qr: "Імпорт QR…",
    scan_qr: "Сканувати QR (камера)",
    scan_camera: "Сканування — наведіть камеру на PeerInvite QR…",
    peer_off_p0: "peer вимкнено (P0)",
    st_stopped: "зупинено",
    st_starting: "запускається",
    st_running: "працює",
    st_unhealthy: "нездоровий",
    st_stopping: "зупиняється",
    st_failed: "збій",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_languages_have_work_tab() {
        assert_eq!(Labels::get(UiLang::En).tab_work, "Work");
        assert_eq!(Labels::get(UiLang::Uk).tab_work, "Робота");
        assert!(Labels::get(UiLang::Uk)
            .open_window_hint
            .contains("не блокує"));
        assert!(Labels::get(UiLang::En)
            .work_not_llm
            .contains("execution-basic"));
        assert!(Labels::get(UiLang::En)
            .work_not_llm
            .contains("text.generate.local"));
        assert!(Labels::get(UiLang::En)
            .work_not_llm
            .contains("never fakes VERIFIED"));
        assert!(Labels::get(UiLang::Uk)
            .work_not_llm
            .contains("text.generate.local"));
        assert!(Labels::get(UiLang::Uk).not_llm.contains("не заборона"));
        assert_eq!(Labels::get(UiLang::Uk).work_answer, "Відповідь");
        assert_eq!(Labels::get(UiLang::En).work_details, "Details");
        assert_eq!(
            Labels::get(UiLang::En).work_execution_id,
            "Execution artifact:"
        );
        assert!(!Labels::get(UiLang::En)
            .work_not_llm
            .to_ascii_lowercase()
            .contains("forbidden"));
        assert!(!Labels::get(UiLang::En)
            .not_llm
            .contains("AIRA is not an LLM runtime"));
    }
}

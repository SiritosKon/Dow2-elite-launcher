// Elite Launcher UI. Draws itself from the status the Rust side returns (get_status) and the
// signed launcher.json inside it. Rule: Play is never blocked by a new version — it is only offered.
'use strict';
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = id => document.getElementById(id);

const T = {
  en: {
    "home": "Home",
    "settings": "Settings",
    "news": "News",
    "version": "Version",
    "links": "Links",
    "play": "Play",
    "install": "Install Elite",
    "updating": "Updating…",
    "installed": "Installed",
    "current": "You have: {v}",
    "notInstalled": "Elite is not installed yet",
    "ready": "Elite {v} is ready",
    "switchTo": "Switch to {name} — download the changes",
    "updateNow": "Update now",
    "checking": "Checking files… {p}%",
    "downloading": "Updating {file}",
    "meta": "Downloaded {dl} · reused from disk {re} · from local cache {fc} · mirror {m}",
    "done": "Done: {n} file(s) updated, downloaded {dl}",
    "upToDate": "Everything is up to date",
    "cancel": "Cancel",
    "cancelled": "Cancelled",
    "connecting": "Connecting to the update server…",
    "offline": "No connection to the update server — you can still play the installed version.",
    "noGame": "Dawn of War II: Retribution not found",
    "noGameText": "Elite Mod is installed on top of the Steam version of the game.",
    "getGame": "Get it on Steam",
    "browse": "Browse…",
    "rescan": "Search again",
    "getSteam": "Install Steam",
    "gameFolder": "Game folder",
    "open": "Open",
    "language": "Language",
    "cache": "Keep previous versions",
    "cacheNote": "switching back to them is instant",
    "mirror": "Extra mirror (optional)",
    "save": "Save",
    "repairTitle": "Repair",
    "repair": "Check & repair files",
    "repairNote": "checks every file and re-downloads only damaged parts",
    "launcherUpdate": "New launcher {v} is available — click to download",
    "starting": "Starting the game…",
    "mirrors": "{n} mirror(s) configured",
    "allNews": "All news",
    "components": "Add-ons",
    "conflictTitle": "This patch is already in your game folder",
    "conflictText": "Found a hand-installed copy: {files}. It conflicts with the launcher version, so the launcher keeps its own copy off while these files are there.",
    "conflictRemove": "Delete these files",
    "conflictKeep": "Leave as is",
    "conflictKeepNote": "Leave it as is and everything will work: the copy in the folder does the job.",
    "conflictActive": "Using the copy from the game folder (dow2-patches) — the launcher version is off",
    "conflictRemoved": "Deleted: {files}",
    "main": "Main",
    "mainNote": "the lobbies play this",
    "active": "Installed now",
    "switchBtn": "Switch",
    "installBtn": "Install",
    "instant": "instant",
    "upTo": "up to {s}",
    "needDl": "needs a download",
    "archive": "Archive versions ({n})",
    "back": "⇄ Back to {name}",
    "youPlay": "You play {name}",
    "notMain": "— the lobbies play {name}",
    "switching": "Switching to {name}…",
    "diagTitle": "Help the developers",
    "diagName": "Your nickname (optional)",
    "sendLogs": "Send logs",
    "sendLogsNote": "sends the logs of the add-ons and the game (lobby filter, observer lag, Continue button) to the developer",
    "logsSent": "Logs sent, thank you! Reference: {id}",
    "sending": "Sending…",
    "sendLogsHome": "📋 Send logs to the developer"
  },
  ru: {
    "home": "Главная",
    "settings": "Настройки",
    "news": "Новости",
    "version": "Версия",
    "links": "Ссылки",
    "play": "Играть",
    "install": "Установить Elite",
    "updating": "Обновление…",
    "installed": "Установлена",
    "current": "У вас: {v}",
    "notInstalled": "Elite ещё не установлен",
    "ready": "Elite {v} готов",
    "switchTo": "Перейти на {name} — скачать изменения",
    "updateNow": "Обновить",
    "checking": "Проверка файлов… {p}%",
    "downloading": "Обновляю {file}",
    "meta": "Скачано {dl} · взято с диска {re} · из локального кэша {fc} · зеркало {m}",
    "done": "Готово: обновлено файлов {n}, скачано {dl}",
    "upToDate": "Всё актуально",
    "cancel": "Отмена",
    "cancelled": "Отменено",
    "connecting": "Подключаюсь к серверу обновлений…",
    "offline": "Нет связи с сервером обновлений — установленную версию можно запускать.",
    "noGame": "Dawn of War II: Retribution не найдена",
    "noGameText": "Elite ставится поверх Steam-версии игры.",
    "getGame": "Купить в Steam",
    "browse": "Указать папку…",
    "rescan": "Искать снова",
    "getSteam": "Установить Steam",
    "gameFolder": "Папка игры",
    "open": "Открыть",
    "language": "Язык",
    "cache": "Хранить прошлые версии",
    "cacheNote": "переключение на них — мгновенно",
    "mirror": "Дополнительное зеркало (необязательно)",
    "save": "Сохранить",
    "repairTitle": "Починка",
    "repair": "Проверить и починить",
    "repairNote": "проверяет все файлы и докачивает только повреждённые части",
    "launcherUpdate": "Вышел новый лаунчер {v} — нажмите, чтобы скачать",
    "starting": "Запускаю игру…",
    "mirrors": "зеркал настроено: {n}",
    "allNews": "Все новости",
    "components": "Дополнения",
    "conflictTitle": "Этот патч уже лежит в папке с игрой",
    "conflictText": "Найдена копия, поставленная вручную: {files}. Она будет конфликтовать с версией из лаунчера, поэтому пока эти файлы на месте, лаунчер свою копию не включает.",
    "conflictRemove": "Удалить эти файлы",
    "conflictKeep": "Оставить как есть",
    "conflictKeepNote": "Если оставить как есть — всё и так будет работать: копия в папке делает то же самое.",
    "conflictActive": "Работает копия из папки игры (dow2-patches) — версия лаунчера отключена",
    "conflictRemoved": "Удалено: {files}",
    "main": "Основная",
    "mainNote": "на ней играют в лобби",
    "active": "Сейчас стоит",
    "switchBtn": "Переключить",
    "installBtn": "Установить",
    "instant": "мгновенно",
    "upTo": "до {s}",
    "needDl": "нужно скачать",
    "archive": "Архивные версии ({n})",
    "back": "⇄ Обратно на {name}",
    "youPlay": "У вас {name}",
    "notMain": "— в лобби играют на {name}",
    "switching": "Переключаю на {name}…",
    "diagTitle": "Помочь разработчикам",
    "diagName": "Ваш ник (необязательно)",
    "sendLogs": "Отправить логи",
    "sendLogsNote": "отправляет разработчику логи дополнений и игры (фильтр лобби, тормоза обсервера, кнопка Continue)",
    "logsSent": "Логи отправлены, спасибо! Номер: {id}",
    "sending": "Отправляю…",
    "sendLogsHome": "📋 Отправить логи разработчику"
  },
  es: {
    "home": "Inicio",
    "settings": "Ajustes",
    "news": "Noticias",
    "version": "Versión",
    "links": "Enlaces",
    "play": "Jugar",
    "install": "Instalar Elite",
    "updating": "Actualizando…",
    "installed": "Instalada",
    "current": "Tienes: {v}",
    "notInstalled": "Elite aún no está instalado",
    "ready": "Elite {v} está listo",
    "switchTo": "Cambiar a {name}: descargar los cambios",
    "updateNow": "Actualizar",
    "checking": "Comprobando archivos… {p}%",
    "downloading": "Actualizando {file}",
    "meta": "Descargado {dl} · reutilizado del disco {re} · de la caché local {fc} · espejo {m}",
    "done": "Listo: {n} archivo(s) actualizados, descargado {dl}",
    "upToDate": "Todo está al día",
    "cancel": "Cancelar",
    "cancelled": "Cancelado",
    "connecting": "Conectando con el servidor de actualizaciones…",
    "offline": "Sin conexión con el servidor de actualizaciones: aún puedes jugar con la versión instalada.",
    "noGame": "No se encontró Dawn of War II: Retribution",
    "noGameText": "Elite Mod se instala sobre la versión de Steam del juego.",
    "getGame": "Conseguirlo en Steam",
    "browse": "Elegir carpeta…",
    "rescan": "Buscar de nuevo",
    "getSteam": "Instalar Steam",
    "gameFolder": "Carpeta del juego",
    "open": "Abrir",
    "language": "Idioma",
    "cache": "Guardar versiones anteriores",
    "cacheNote": "volver a ellas es instantáneo",
    "mirror": "Espejo adicional (opcional)",
    "save": "Guardar",
    "repairTitle": "Reparar",
    "repair": "Comprobar y reparar archivos",
    "repairNote": "comprueba cada archivo y vuelve a descargar solo las partes dañadas",
    "launcherUpdate": "Hay un nuevo launcher {v}: haz clic para descargarlo",
    "starting": "Iniciando el juego…",
    "mirrors": "{n} espejo(s) configurados",
    "allNews": "Todas las noticias",
    "components": "Complementos",
    "conflictTitle": "Este parche ya está en la carpeta del juego",
    "conflictText": "Se encontró una copia instalada a mano: {files}. Entra en conflicto con la versión del launcher, así que el launcher no activa la suya mientras estos archivos estén ahí.",
    "conflictRemove": "Borrar estos archivos",
    "conflictKeep": "Dejar como está",
    "conflictKeepNote": "Si lo dejas como está, todo funcionará igual: la copia de la carpeta hace el mismo trabajo.",
    "conflictActive": "Se usa la copia de la carpeta del juego (dow2-patches); la versión del launcher está desactivada",
    "conflictRemoved": "Borrado: {files}",
    "main": "Principal",
    "mainNote": "se juega en los lobbies",
    "active": "Instalada",
    "switchBtn": "Cambiar",
    "installBtn": "Instalar",
    "instant": "instantáneo",
    "upTo": "hasta {s}",
    "needDl": "requiere descarga",
    "archive": "Versiones archivadas ({n})",
    "back": "⇄ Volver a {name}",
    "youPlay": "Juegas {name}",
    "notMain": "— en los lobbies se juega {name}",
    "switching": "Cambiando a {name}…",
    "diagTitle": "Ayuda a los desarrolladores",
    "diagName": "Tu nick (opcional)",
    "sendLogs": "Enviar registros",
    "sendLogsNote": "envía al desarrollador los registros de los complementos y del juego (filtro del lobby, lag del observador, botón Continue)",
    "logsSent": "Registros enviados, ¡gracias! Referencia: {id}",
    "sending": "Enviando…",
    "sendLogsHome": "📋 Enviar registros al desarrollador"
  },
  fr: {
    "home": "Accueil",
    "settings": "Paramètres",
    "news": "Actualités",
    "version": "Version",
    "links": "Liens",
    "play": "Jouer",
    "install": "Installer Elite",
    "updating": "Mise à jour…",
    "installed": "Installée",
    "current": "Vous avez : {v}",
    "notInstalled": "Elite n'est pas encore installé",
    "ready": "Elite {v} est prêt",
    "switchTo": "Passer à {name} — télécharger les changements",
    "updateNow": "Mettre à jour",
    "checking": "Vérification des fichiers… {p} %",
    "downloading": "Mise à jour de {file}",
    "meta": "Téléchargé {dl} · réutilisé du disque {re} · depuis le cache local {fc} · miroir {m}",
    "done": "Terminé : {n} fichier(s) mis à jour, téléchargé {dl}",
    "upToDate": "Tout est à jour",
    "cancel": "Annuler",
    "cancelled": "Annulé",
    "connecting": "Connexion au serveur de mises à jour…",
    "offline": "Pas de connexion au serveur de mises à jour — vous pouvez quand même jouer à la version installée.",
    "noGame": "Dawn of War II: Retribution introuvable",
    "noGameText": "Elite Mod s'installe par-dessus la version Steam du jeu.",
    "getGame": "L'obtenir sur Steam",
    "browse": "Choisir le dossier…",
    "rescan": "Rechercher à nouveau",
    "getSteam": "Installer Steam",
    "gameFolder": "Dossier du jeu",
    "open": "Ouvrir",
    "language": "Langue",
    "cache": "Garder les versions précédentes",
    "cacheNote": "y revenir est instantané",
    "mirror": "Miroir supplémentaire (facultatif)",
    "save": "Enregistrer",
    "repairTitle": "Réparer",
    "repair": "Vérifier et réparer les fichiers",
    "repairNote": "vérifie chaque fichier et ne retélécharge que les parties endommagées",
    "launcherUpdate": "Nouveau launcher {v} disponible — cliquez pour le télécharger",
    "starting": "Lancement du jeu…",
    "mirrors": "{n} miroir(s) configuré(s)",
    "allNews": "Toutes les actualités",
    "components": "Modules",
    "conflictTitle": "Ce patch est déjà dans le dossier du jeu",
    "conflictText": "Une copie installée à la main a été trouvée : {files}. Elle entre en conflit avec la version du launcher, donc le launcher garde la sienne désactivée tant que ces fichiers sont là.",
    "conflictRemove": "Supprimer ces fichiers",
    "conflictKeep": "Laisser tel quel",
    "conflictKeepNote": "Si vous laissez tel quel, tout fonctionnera : la copie du dossier fait le même travail.",
    "conflictActive": "La copie du dossier du jeu (dow2-patches) est utilisée — la version du launcher est désactivée",
    "conflictRemoved": "Supprimé : {files}",
    "main": "Principale",
    "mainNote": "c'est celle des lobbies",
    "active": "Installée",
    "switchBtn": "Changer",
    "installBtn": "Installer",
    "instant": "instantané",
    "upTo": "jusqu'à {s}",
    "needDl": "téléchargement nécessaire",
    "archive": "Versions archivées ({n})",
    "back": "⇄ Revenir à {name}",
    "youPlay": "Vous jouez {name}",
    "notMain": "— les lobbies jouent {name}",
    "switching": "Passage à {name}…",
    "diagTitle": "Aider les développeurs",
    "diagName": "Votre pseudo (facultatif)",
    "sendLogs": "Envoyer les journaux",
    "sendLogsNote": "envoie au développeur les journaux des modules et du jeu (filtre du lobby, lag de l'observateur, bouton Continue)",
    "logsSent": "Journaux envoyés, merci ! Référence : {id}",
    "sending": "Envoi…",
    "sendLogsHome": "📋 Envoyer les journaux au développeur"
  },
  de: {
    "home": "Start",
    "settings": "Einstellungen",
    "news": "Neuigkeiten",
    "version": "Version",
    "links": "Links",
    "play": "Spielen",
    "install": "Elite installieren",
    "updating": "Aktualisiere…",
    "installed": "Installiert",
    "current": "Installiert: {v}",
    "notInstalled": "Elite ist noch nicht installiert",
    "ready": "Elite {v} ist bereit",
    "switchTo": "Zu {name} wechseln — Änderungen herunterladen",
    "updateNow": "Aktualisieren",
    "checking": "Prüfe Dateien… {p} %",
    "downloading": "Aktualisiere {file}",
    "meta": "Heruntergeladen {dl} · von der Festplatte übernommen {re} · aus dem lokalen Cache {fc} · Mirror {m}",
    "done": "Fertig: {n} Datei(en) aktualisiert, {dl} heruntergeladen",
    "upToDate": "Alles ist aktuell",
    "cancel": "Abbrechen",
    "cancelled": "Abgebrochen",
    "connecting": "Verbinde mit dem Update-Server…",
    "offline": "Keine Verbindung zum Update-Server — die installierte Version kannst du trotzdem spielen.",
    "noGame": "Dawn of War II: Retribution nicht gefunden",
    "noGameText": "Elite Mod wird über der Steam-Version des Spiels installiert.",
    "getGame": "Auf Steam holen",
    "browse": "Ordner wählen…",
    "rescan": "Erneut suchen",
    "getSteam": "Steam installieren",
    "gameFolder": "Spielordner",
    "open": "Öffnen",
    "language": "Sprache",
    "cache": "Frühere Versionen behalten",
    "cacheNote": "zurückwechseln geht sofort",
    "mirror": "Zusätzlicher Mirror (optional)",
    "save": "Speichern",
    "repairTitle": "Reparieren",
    "repair": "Dateien prüfen und reparieren",
    "repairNote": "prüft jede Datei und lädt nur beschädigte Teile neu",
    "launcherUpdate": "Neuer Launcher {v} verfügbar — zum Herunterladen klicken",
    "starting": "Spiel wird gestartet…",
    "mirrors": "{n} Mirror(s) eingerichtet",
    "allNews": "Alle Neuigkeiten",
    "components": "Erweiterungen",
    "conflictTitle": "Dieser Patch liegt schon im Spielordner",
    "conflictText": "Eine von Hand installierte Kopie wurde gefunden: {files}. Sie kollidiert mit der Launcher-Version, daher lässt der Launcher seine eigene aus, solange diese Dateien da sind.",
    "conflictRemove": "Diese Dateien löschen",
    "conflictKeep": "So lassen",
    "conflictKeepNote": "Wenn du es so lässt, funktioniert trotzdem alles: die Kopie im Ordner erledigt dasselbe.",
    "conflictActive": "Die Kopie aus dem Spielordner (dow2-patches) wird verwendet – die Launcher-Version ist aus",
    "conflictRemoved": "Gelöscht: {files}",
    "main": "Haupt",
    "mainNote": "diese Version spielen die Lobbys",
    "active": "Jetzt installiert",
    "switchBtn": "Wechseln",
    "installBtn": "Installieren",
    "instant": "sofort",
    "upTo": "bis {s}",
    "needDl": "Download nötig",
    "archive": "Archivversionen ({n})",
    "back": "⇄ Zurück zu {name}",
    "youPlay": "Du spielst {name}",
    "notMain": "— die Lobbys spielen {name}",
    "switching": "Wechsle zu {name}…",
    "diagTitle": "Den Entwicklern helfen",
    "diagName": "Dein Nickname (optional)",
    "sendLogs": "Logs senden",
    "sendLogsNote": "schickt dem Entwickler die Logs der Erweiterungen und des Spiels (Lobby-Filter, Beobachter-Lag, Continue-Knopf)",
    "logsSent": "Logs gesendet, danke! Nummer: {id}",
    "sending": "Sende…",
    "sendLogsHome": "📋 Logs an den Entwickler senden"
  },
};

let S = null;          // last status from Rust
let lang = 'en';
let prog = null;       // last progress event

const t = (k, vars = {}) => (T[lang][k] || T.en[k] || k).replace(/\{(\w+)\}/g, (_, v) => vars[v] ?? '');
const L = x => x == null ? '' : typeof x === 'string' ? x : (x[lang] || x.en || Object.values(x)[0] || '');
const mb = b => b >= 1e9 ? (b / 1e9).toFixed(2) + ' GB' : (b / 1e6).toFixed(b < 1e7 ? 2 : 1) + ' MB';
const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const open = url => url && invoke('open_url', { url }).catch(e => toast(e, true));

function toast(msg, err) {
  const el = $('toast'); el.textContent = String(msg); el.className = 'toast' + (err ? ' err' : '');
  clearTimeout(toast.t); toast.t = setTimeout(() => el.classList.add('hidden'), 4500);
}

function channelOf(id) { return (S.config?.channels || []).find(c => c.id === id); }

function render() {
  lang = S.lang || (navigator.language || 'en').slice(0, 2);
  if (!T[lang]) lang = 'en';
  document.documentElement.lang = lang;
  document.querySelectorAll('[data-t]').forEach(e => e.textContent = t(e.dataset.t));
  document.querySelectorAll('[data-tp]').forEach(e => e.placeholder = t(e.dataset.tp));
  const cfg = S.config || {};
  $('title').textContent = L(cfg.title) || 'Elite Mod';
  $('subtitle').textContent = L(cfg.subtitle) || 'Dawn of War II · Retribution';

  // launcher self-update: offered, never forced
  const lu = S.launcher_update;
  $('launcherBanner').classList.toggle('hidden', !lu);
  if (lu) { $('launcherBanner').textContent = L(lu.text) || t('launcherUpdate', { v: lu.version }); $('launcherBanner').onclick = () => open(lu.url); }

  // announcements
  const today = new Date().toISOString().slice(0, 10);
  $('announcements').innerHTML = (cfg.announcements || [])
    .filter(a => !(S.dismissed || []).includes(a.id) && (!a.until || a.until >= today))
    .map(a => `<div class="ann ${esc(a.style)}" data-url="${esc(a.url)}"><span class="tx">${esc(L(a.text))}</span>${a.id ? `<span class="cl" data-id="${esc(a.id)}">✕</span>` : ''}</div>`).join('');
  $('announcements').querySelectorAll('.ann').forEach(el => el.onclick = e => {
    if (e.target.dataset.id) { invoke('set_setting', { key: 'dismiss', value: e.target.dataset.id }).then(reload); return; }
    open(el.dataset.url);
  });

  // news
  $('news').innerHTML = (cfg.news || []).map((n, i) => `<div class="item" data-i="${i}"><div class="d">${esc(n.date)}</div><div class="t">${esc(L(n.title))}</div><div class="x">${esc(L(n.text))}</div></div>`).join('')
    || `<div class="installed">${esc(S.config_loaded ? '' : t('connecting'))}</div>`;
  $('news').querySelectorAll('.item').forEach(el => el.onclick = () => open(cfg.news[+el.dataset.i].url));
  const more = cfg.news_more;
  $('newsMore').classList.toggle('hidden', !more?.url);
  if (more?.url) { $('newsMore').textContent = L(more.title) || t('allNews'); $('newsMore').onclick = () => open(more.url); }

  // game / steam missing → guide the player
  const setup = $('setup');
  if (!S.game_found) {
    const g = cfg.game || {};
    setup.innerHTML = `<h2>${esc(t('noGame'))}</h2><p>${esc(L(g.requirements) || t('noGameText'))}</p><div class="row">
      ${S.steam_found ? '' : `<button class="btn red" id="getSteam">${esc(t('getSteam'))}</button>`}
      <button class="btn gold" id="getGame">${esc(t('getGame'))}</button>
      <button class="btn" id="pick">${esc(t('browse'))}</button><button class="btn" id="rescan">${esc(t('rescan'))}</button></div>`;
    setup.classList.remove('hidden');
    $('getGame').onclick = () => open(g.store_url || 'steam://store/56400');
    if ($('getSteam')) $('getSteam').onclick = () => open('https://store.steampowered.com/about/');
    $('pick').onclick = pickDir; $('rescan').onclick = reload;
  } else setup.classList.add('hidden');

  // versions: main one marked, the installed one marked, every other one switches in one click
  const chans = cfg.channels || [];
  const activeCh = chans.find(c => c.version === S.installed_version && c.id === S.installed_channel) || chans.find(c => c.version === S.installed_version);
  const mainCh = chans.find(c => c.id === S.main_channel);
  const row = c => {
    const isActive = activeCh && c.id === activeCh.id, isMain = c.id === S.main_channel;
    const cost = (S.costs || {})[c.version];
    const hint = cost === undefined ? t('needDl') : cost === 0 ? t('instant') : t('upTo', { s: mb(cost) });
    const tags = (isMain ? `<span class="tag main">${esc(t('main'))}</span>` : '') + (L(c.badge) && !isMain ? `<span class="tag beta">${esc(L(c.badge))}</span>` : '');
    const right = isActive ? `<span class="tag ok">✓ ${esc(t('active'))}</span>`
      : `<button class="btn sw" data-id="${esc(c.id)}" ${S.busy || !S.game_found || !S.config_loaded ? 'disabled' : ''}>${esc(S.installed_version ? t('switchBtn') : t('installBtn'))}<small>${esc(hint)}</small></button>`;
    return `<div class="ch ${isActive ? 'act' : ''} ${isMain ? 'main' : ''}"><div class="nm"><b>${esc(L(c.name) || c.id)}</b>${tags}
      <small>${esc(isMain ? t('mainNote') : L(c.description))}</small></div>${right}</div>`;
  };
  const live = chans.filter(c => !c.archive), arch = chans.filter(c => c.archive);
  $('channels').innerHTML = live.map(row).join('') + (arch.length ? `<details class="arch" ${arch.some(c => activeCh && c.id === activeCh.id) ? 'open' : ''}><summary>${esc(t('archive', { n: arch.length }))}</summary>${arch.map(row).join('')}</details>` : '');
  $('channels').querySelectorAll('.sw').forEach(b => b.onclick = () => switchTo(b.dataset.id));
  $('installed').innerHTML = activeCh
    ? esc(t('youPlay', { name: L(activeCh.name) })) + (mainCh && activeCh.id !== mainCh.id ? ' <span class="warn">' + esc(t('notMain', { name: L(mainCh.name) })) + '</span>' : '')
    : esc(S.installed_version ? t('current', { v: S.installed_version }) : (S.game_found ? t('notInstalled') : ''));

  // add-ons (components)
  const vshort = (c, v) => String(v || '').replace(new RegExp('^' + c.id + '-'), '');
  const compVer = c => !c.installed ? (c.enabled && c.latest ? ' <span class="tag">→ v' + esc(vshort(c, c.latest)) + '</span>' : '')
    : ' <span class="tag ok">v' + esc(vshort(c, c.installed)) + '</span>'
      + (c.latest && c.latest !== c.installed ? ' <span class="tag">→ v' + esc(vshort(c, c.latest)) + '</span>' : '');
  const comps = S.components || [];
  $('compBox').classList.toggle('hidden', !comps.length);
  $('components').innerHTML = comps.map(c => `<label class="comp"><input type="checkbox" data-id="${esc(c.id)}" ${c.enabled ? 'checked' : ''} ${S.busy ? 'disabled' : ''}><div><b>${esc(L(c.name) || c.id)}${compVer(c)}</b><small>${esc(L(c.description))}</small>${c.enabled && (c.conflicts || []).length ? `<small class="warn">${esc(t('conflictActive'))}</small>` : ''}</div></label>`
    + (c.options || []).map(o => `<label class="opt"><input type="checkbox" data-comp="${esc(c.id)}" data-opt="${esc(o.id)}" ${o.on ? 'checked' : ''} ${!c.enabled || S.busy ? 'disabled' : ''}><span>${esc(L(o.name) || o.id)}</span></label>`).join('')).join('');
  $('components').querySelectorAll('input[data-opt]').forEach(cb => cb.onchange = () =>
    invoke('set_comp_option', { comp: cb.dataset.comp, opt: cb.dataset.opt, on: cb.checked }).then(reload).catch(e => toast(e, true)));
  $('components').querySelectorAll('input[data-id]').forEach(cb => cb.onchange = async () => {
    const comp = comps.find(c => c.id === cb.dataset.id);
    if (cb.checked && comp && (comp.conflicts || []).length) {
      const del = await askConflict(comp.conflicts);
      if (del) {
        try { toast(t('conflictRemoved', { files: (await invoke('remove_conflicts', { id: comp.id })).join(', ') })); }
        catch (e) { toast(e, true); }
      }
    }
    await invoke('set_component', { id: cb.dataset.id, on: cb.checked });
    if (S.installed_version) { try { await invoke('start_update', { repair: false, componentsOnly: true }); S.busy = true; render(); } catch (e) { toast(e, true); } }
    else reload();
  });
  // links
  $('links').innerHTML = (cfg.links || []).map((l, i) => `<button data-i="${i}">${esc(L(l.title))}</button>`).join('');
  $('links').querySelectorAll('button').forEach(b => b.onclick = () => open(cfg.links[+b.dataset.i].url));

  // update banner (non-blocking)
  const target = channelOf(S.installed_channel) || channelOf(S.channel);
  const showUpd = target && S.installed_version && target.version !== S.installed_version && !S.busy;
  $('updateBanner').classList.toggle('hidden', !showUpd);
  if (showUpd) {
    $('updateText').textContent = (L(cfg.update_text) || t('switchTo', { name: '{name}' })).replace('{name}', L(target.name)).replace('{version}', target.version);
    $('updateBtn').textContent = t('updateNow');
    $('updateBtn').onclick = () => switchTo(target.id);
  }

  // footer
  const play = $('play');
  play.classList.remove('install');
  $('cancel').classList.toggle('hidden', !S.busy);
  if (S.busy) { play.textContent = t('updating'); play.classList.add('install'); play.disabled = true; }
  else if (!S.game_found) { play.textContent = t('play'); play.disabled = true; }
  else if (!S.installed_version) { play.textContent = t('install'); play.classList.add('install'); play.disabled = !S.config_loaded; }
  else { play.textContent = t('play'); play.disabled = false; }
  play.onclick = () => (S.installed_version ? doPlay() : switchTo(S.main_channel || S.channel));
  $('playWhat').textContent = S.busy ? '' : activeCh ? L(activeCh.name) : (!S.installed_version && mainCh ? L(mainCh.name) : '');
  const prev = channelOf(S.prev_channel || '');
  $('quick').classList.toggle('hidden', !prev || S.busy || !S.installed_version);
  if (prev) { $('quick').textContent = t('back', { name: L(prev.name) }); $('quick').onclick = () => switchTo(prev.id); }

  if (!S.busy) {
    $('bar').classList.add('hidden');
    $('statusText').innerHTML = S.last_error && !S.config_loaded ? esc(t('offline'))
      : S.last_error ? esc(S.last_error)
      : S.installed_version ? esc(t('ready', { v: activeCh ? L(activeCh.name).replace(/^Elite\s+/i, '') : S.installed_version })) : (S.config_loaded ? '' : esc(t('connecting')));
    const hint = L(cfg.launch?.[S.os]?.hint);
    $('statusMeta').textContent = [hint, S.game_dir || ''].filter(Boolean).join(' · ');
  }

  // settings
  $('gameDir').textContent = S.game_dir || '—';
  $('lang').value = S.lang || '';
  $('cache').value = String(S.cache_mb);
  $('mirror').value = S.extra_mirror || '';
  $('mirrorInfo').textContent = t('mirrors', { n: S.mirror_count });
  $('repair').disabled = S.busy || !S.installed_version;
  $('about').textContent = `Elite Launcher ${S.launcher_version} · ${S.os}`;
}

function renderProgress(p) {
  $('bar').classList.remove('hidden');
  const pct = p.bytes_total ? Math.min(100, Math.round(100 * p.bytes_done / p.bytes_total)) : 0;
  $('barFill').style.width = pct + '%';
  $('statusText').innerHTML = p.phase === 'checking' ? esc(t('checking', { p: pct })) : esc(t('downloading', { file: p.file.split('/').pop() })) + ` <b>${pct}%</b>`;
  $('statusMeta').textContent = t('meta', { dl: mb(p.downloaded), re: mb(p.reused), fc: mb(p.from_cache), m: p.mirror });
}

// the add-on is already installed by hand (stand-alone dow2-patches package): delete those files or keep them
function askConflict(files) {
  return new Promise(done => {
    const m = document.createElement('div');
    m.className = 'modal';
    m.innerHTML = `<div class="box"><h3>${esc(t('conflictTitle'))}</h3><p>${esc(t('conflictText', { files: files.join(', ') }))}</p>
      <p class="note">${esc(t('conflictKeepNote'))}</p><div class="row"><button class="btn red" id="cfDel">${esc(t('conflictRemove'))}</button>
      <button class="btn gold" id="cfKeep">${esc(t('conflictKeep'))}</button></div></div>`;
    document.body.appendChild(m);
    const close = v => { m.remove(); done(v); };
    m.querySelector('#cfDel').onclick = () => close(true);
    m.querySelector('#cfKeep').onclick = () => close(false);
  });
}

async function reload() { S = await invoke('get_status'); render(); }
async function refresh() {
  try { S = await invoke('refresh'); render(); } catch (e) { toast(e, true); return; }
  // add-ons follow their checkboxes on their own: install missing ones, remove unticked ones
  const comps = S.components || [];
  if (S.installed_version && !S.busy && comps.some(c => (c.enabled && !(c.conflicts || []).length) !== !!c.installed)) {
    try { await invoke('start_update', { repair: false, componentsOnly: true }); S.busy = true; render(); } catch (e) { /* will retry next refresh */ }
  }
}
async function pickDir() { try { S = await invoke('pick_game_dir'); render(); } catch (e) { toast(e, true); } }

async function switchTo(id) {
  const c = channelOf(id);
  try { await invoke('switch_to', { id }); S.busy = true; render(); if (c) $('statusText').textContent = t('switching', { name: L(c.name) }); }
  catch (e) { toast(e, true); }
}

async function startUpdate(repair) {
  try { await invoke('start_update', { repair }); S.busy = true; render(); }
  catch (e) { toast(e, true); }
}

async function doPlay() {
  try { await invoke('play'); toast(t('starting')); } catch (e) { toast(e, true); }
}

listen('progress', e => { prog = e.payload; if (S) { S.busy = true; renderProgress(prog); } });
listen('update_done', async e => {
  await reload();
  const r = e.payload;
  if (r.ok) toast(r.summary.updated ? t('done', { n: r.summary.updated, dl: mb(r.summary.downloaded) }) : t('upToDate'));
  else toast(r.cancelled ? t('cancelled') : r.error, !r.cancelled);
});

document.querySelectorAll('.tab').forEach(b => b.onclick = () => {
  document.querySelectorAll('.tab').forEach(x => x.classList.toggle('on', x === b));
  document.querySelectorAll('.view').forEach(v => v.classList.toggle('hidden', v.id !== b.dataset.view));
});
$('cancel').onclick = () => invoke('cancel_update');
$('browse').onclick = pickDir;
$('openDir').onclick = () => invoke('open_game_dir').catch(e => toast(e, true));
$('lang').onchange = e => invoke('set_setting', { key: 'lang', value: e.target.value }).then(reload);
$('cache').onchange = e => invoke('set_setting', { key: 'cache_mb', value: +e.target.value }).then(reload);
$('saveMirror').onclick = () => invoke('set_setting', { key: 'extra_mirror', value: $('mirror').value.trim() }).then(refresh);
$('repair').onclick = () => startUpdate(true);
// ник для логов запоминаем (localStorage может быть недоступен — тогда просто без ника)
try { $('diagName').value = localStorage.getItem('diagName') || ''; } catch (e) {}
$('diagName').onchange = () => { try { localStorage.setItem('diagName', $('diagName').value.trim()); } catch (e) {} };
async function sendLogs(btn, label) {
  btn.disabled = true; btn.textContent = t('sending');
  try { const id = await invoke('send_logs', { who: $('diagName').value.trim() }); toast(t('logsSent', { id })); }
  catch (e) { toast(e, true); }
  finally { btn.disabled = false; btn.textContent = t(label); }
}
$('sendLogs').onclick = () => sendLogs($('sendLogs'), 'sendLogs');
$('sendLogsHome').onclick = () => sendLogs($('sendLogsHome'), 'sendLogsHome');

reload().then(refresh);
setInterval(() => { if (S && !S.busy) refresh(); }, 15 * 60 * 1000);   // pick up new versions and announcements

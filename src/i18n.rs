use crate::config::Language;

// Keep Russian as the source text and fallback, with language order matching Language::index.
const TEXT: &[(&str, [&str; 8])] = &[
    ("Датчики", ["Датчики", "Sensors", "Sensores", "Sensores", "Sensores", "Capteurs", "Sensori", "传感器"]),
    ("Расположение", ["Расположение", "Layout", "Layout", "چیدمان", "Distribución", "Disposition", "Layout", "布局"]),
    ("Общие", ["Общие", "General", "Geral", "عمومی", "General", "Général", "Generale", "常规"]),
    ("Изменения автоматически сохранены рядом с приложением", ["Изменения автоматически сохранены рядом с приложением", "Changes saved beside the app", "Alterações salvas ao lado do aplicativo", "تغییرات کنار برنامه ذخیره شد", "Cambios guardados junto a la aplicación", "Modifications enregistrées à côté de l’application", "Modifiche salvate accanto all’applicazione", "更改已保存到应用程序旁边"]),
    ("Датчики и обнаруженное оборудование", ["Датчики и обнаруженное оборудование", "Sensors and detected hardware", "Sensores e hardware detectado", "حسگرها و سخت‌افزار شناسایی‌شده", "Sensores y hardware detectado", "Capteurs et matériel détecté", "Sensori e hardware rilevato", "传感器与已检测硬件"]),
    ("Оборудование определяется автоматически; выберите блоки и устройства для панели задач.", ["Оборудование определяется автоматически; выберите блоки и устройства для панели задач.", "Hardware is detected automatically; choose the sections and devices to show in the taskbar.", "O hardware é detectado automaticamente; escolha as seções e os dispositivos da barra de tarefas.", "سخت‌افزار به‌طور خودکار شناسایی می‌شود؛ بخش‌ها و دستگاه‌های نوار وظیفه را انتخاب کنید.", "El hardware se detecta automáticamente; elige las secciones y dispositivos de la barra de tareas.", "Le matériel est détecté automatiquement ; choisissez les sections et appareils à afficher dans la barre des tâches.", "L’hardware viene rilevato automaticamente; scegli le sezioni e i dispositivi da mostrare nella barra delle applicazioni.", "硬件会自动检测；选择要在任务栏显示的模块和设备。"]),
    ("Провайдер датчиков: {}", ["Провайдер датчиков: {}", "Sensor provider: {}", "Provedor de sensores: {}", "ارائه‌دهنده حسگر: {}", "Proveedor de sensores: {}", "Fournisseur de capteurs : {}", "Provider sensori: {}", "传感器提供程序：{}"]),
    ("Источник температуры: {}", ["Источник температуры: {}", "Temperature source: {}", "Origem da temperatura: {}", "منبع دما: {}", "Fuente de temperatura: {}", "Source de température : {}", "Fonte della temperatura: {}", "温度来源：{}"]),
    ("ожидание данных", ["ожидание данных", "waiting for data", "aguardando dados", "در انتظار داده", "esperando datos", "en attente de données", "in attesa dei dati", "等待数据"]),
    ("Материнская плата не обнаружена", ["Материнская плата не обнаружена", "Motherboard not detected", "Placa-mãe não detectada", "مادربرد شناسایی نشد", "Placa base no detectada", "Carte mère non détectée", "Scheda madre non rilevata", "未检测到主板"]),
    ("Диски", ["Диски", "Disks", "Discos", "دیسک‌ها", "Discos", "Disques", "Dischi", "磁盘"]),
    ("Скорость суммы:", ["Скорость суммы:", "Aggregate speed:", "Velocidade total:", "سرعت مجموع:", "Velocidad total:", "Débit total :", "Velocità totale:", "总速度："]),
    ("Сеть (суммарно)", ["Сеть (суммарно)", "Network (aggregate)", "Rede (total)", "شبکه (مجموع)", "Red (total)", "Réseau (total)", "Rete (totale)", "网络（合计）"]),
    ("Модель не обнаружена", ["Модель не обнаружена", "Model not detected", "Modelo não detectado", "مدلی شناسایی نشد", "Modelo no detectado", "Modèle non détecté", "Modello non rilevato", "未检测到模型"]),
    ("{} · {} потоков · {:.0}% · {} MHz · температура {}", ["{} · {} потоков · {:.0}% · {} MHz · температура {}", "{} · {} threads · {:.0}% · {} MHz · temperature {}", "{} · {} threads · {:.0}% · {} MHz · temperatura {}", "{} · {} رشته · {:.0}% · {} مگاهرتز · دما {}", "{} · {} hilos · {:.0}% · {} MHz · temperatura {}", "{} · {} threads · {:.0}% · {} MHz · température {}", "{} · {} thread · {:.0}% · {} MHz · temperatura {}", "{} · {} 线程 · {:.0}% · {} MHz · 温度 {}"]),
    ("{} · загрузка {} · VRAM {}/{} MiB · температура {}", ["{} · загрузка {} · VRAM {}/{} MiB · температура {}", "{} · load {} · VRAM {}/{} MiB · temperature {}", "{} · uso {} · VRAM {}/{} MiB · temperatura {}", "{} · بار {} · حافظه ویدئویی {}/{} مگابایت · دما {}", "{} · uso {} · VRAM {}/{} MiB · temperatura {}", "{} · charge {} · VRAM {}/{} Mio · température {}", "{} · utilizzo {} · VRAM {}/{} MiB · temperatura {}", "{} · 负载 {} · 显存 {}/{} MiB · 温度 {}"]),
    ("{} {}{:.1}/{:.1} GiB", ["{} {}{:.1}/{:.1} GiB", "{} {}{:.1}/{:.1} GiB", "{} {}{:.1}/{:.1} GiB", "{} {}{:.1}/{:.1} GiB", "{} {}{:.1}/{:.1} GiB", "{} {}{:.1}/{:.1} Gio", "{} {}{:.1}/{:.1} GiB", "{} {}{:.1}/{:.1} GiB"]),
    ("Диски", ["Диски", "Disks", "Discos", "دیسک‌ها", "Discos", "Disques", "Dischi", "磁盘"]),
    ("{} · {:.1} GiB · чтение {:.2} {} · запись {:.2} {}", ["{} · {:.1} GiB · чтение {:.2} {} · запись {:.2} {}", "{} · {:.1} GiB · read {:.2} {} · write {:.2} {}", "{} · {:.1} GiB · leitura {:.2} {} · gravação {:.2} {}", "{} · {:.1} گیبی‌بایت · خواندن {:.2} {} · نوشتن {:.2} {}", "{} · {:.1} GiB · lectura {:.2} {} · escritura {:.2} {}", "{} · {:.1} Gio · lecture {:.2} {} · écriture {:.2} {}", "{} · {:.1} GiB · lettura {:.2} {} · scrittura {:.2} {}", "{} · {:.1} GiB · 读取 {:.2} {} · 写入 {:.2} {}"]),
    ("{} · {} · {}", ["{} · {} · {}", "{} · {} · {}", "{} · {} · {}", "{} · {} · {}", "{} · {} · {}", "{} · {} · {}", "{} · {} · {}", "{} · {} · {}"]),
    ("токен/с", ["токен/с", "token/s", "token/s", "توکن/ثانیه", "token/s", "jeton/s", "token/s", "令牌/秒"]),
    ("Порядок показателей в панели задач", ["Порядок показателей в панели задач", "Taskbar metric order", "Ordem das métricas na barra de tarefas", "ترتیب شاخص‌ها در نوار وظیفه", "Orden de métricas en la barra de tareas", "Ordre des mesures dans la barre des tâches", "Ordine delle metriche nella barra delle applicazioni", "任务栏指标顺序"]),
    ("Здесь показаны только включённые блоки и устройства. Порядок сохраняется рядом с EXE.", ["Здесь показаны только включённые блоки и устройства. Порядок сохраняется рядом с EXE.", "Only enabled sections and devices are shown here. The order is saved next to the EXE.", "Somente seções e dispositivos ativos são exibidos. A ordem é salva ao lado do EXE.", "فقط بخش‌ها و دستگاه‌های فعال نمایش داده می‌شوند. ترتیب کنار فایل EXE ذخیره می‌شود.", "Aquí solo se muestran las secciones y dispositivos activos. El orden se guarda junto al EXE.", "Seuls les sections et appareils activés sont affichés. L’ordre est enregistré à côté de l’EXE.", "Sono mostrati solo sezioni e dispositivi attivi. L’ordine viene salvato accanto all’EXE.", "这里只显示已启用的模块和设备。顺序保存在 EXE 旁边。"]),
    ("Отступ между колонками", ["Отступ между колонками", "Column spacing", "Espaçamento entre colunas", "فاصله بین ستون‌ها", "Espaciado entre columnas", "Espacement des colonnes", "Spaziatura colonne", "列间距"]),
    ("Пользовательские группы", ["Пользовательские группы", "Custom groups", "Grupos personalizados", "گروه‌های سفارشی", "Grupos personalizados", "Groupes personnalisés", "Gruppi personalizzati", "自定义分组"]),
    ("В каждой группе выберите верхнюю и нижнюю строку из включённых датчиков.", ["В каждой группе выберите верхнюю и нижнюю строку из включённых датчиков.", "Choose a top and bottom row from enabled sensors for each group.", "Escolha uma linha superior e inferior entre os sensores ativos para cada grupo.", "برای هر گروه، ردیف بالا و پایین را از حسگرهای فعال انتخاب کنید.", "Elige una fila superior e inferior entre los sensores activos de cada grupo.", "Choisissez une ligne supérieure et une ligne inférieure parmi les capteurs activés.", "Scegli una riga superiore e una inferiore tra i sensori attivi per ogni gruppo.", "为每个分组从已启用的传感器中选择上、下两行。"]),
    ("Группа {}", ["Группа {}", "Group {}", "Grupo {}", "گروه {}", "Grupo {}", "Groupe {}", "Gruppo {}", "分组 {}"]),
    ("Удалить", ["Удалить", "Remove", "Remover", "حذف", "Eliminar", "Supprimer", "Rimuovi", "删除"]),
    ("Верхняя строка", ["Верхняя строка", "Top row", "Linha superior", "ردیف بالا", "Fila superior", "Ligne supérieure", "Riga superiore", "上行"]),
    ("Нижняя строка", ["Нижняя строка", "Bottom row", "Linha inferior", "ردیف پایین", "Fila inferior", "Ligne inférieure", "Riga inferiore", "下行"]),
    ("Ширина группы (символов)", ["Ширина группы (символов)", "Group width (characters)", "Largura do grupo (caracteres)", "عرض گروه (نویسه)", "Ancho del grupo (caracteres)", "Largeur du groupe (caractères)", "Larghezza gruppo (caratteri)", "分组宽度（字符）"]),
    ("Добавить группу", ["Добавить группу", "Add group", "Adicionar grupo", "افزودن گروه", "Añadir grupo", "Ajouter un groupe", "Aggiungi gruppo", "添加分组"]),
    ("Нет включённых блоков для панели задач.", ["Нет включённых блоков для панели задач.", "No enabled sections for the taskbar.", "Não há seções ativas para a barra de tarefas.", "هیچ بخشی برای نوار وظیفه فعال نیست.", "No hay secciones activas para la barra de tareas.", "Aucune section activée pour la barre des tâches.", "Nessuna sezione attiva per la barra delle applicazioni.", "任务栏没有已启用的模块。"]),
    ("Настройки LM Studio", ["Настройки LM Studio", "LM Studio settings", "Configurações do LM Studio", "تنظیمات LM Studio", "Ajustes de LM Studio", "Paramètres de LM Studio", "Impostazioni LM Studio", "LM Studio 设置"]),
    ("Название блока", ["Название блока", "Section name", "Nome da seção", "نام بخش", "Nombre de sección", "Nom de section", "Nome sezione", "模块名称"]),
    ("Адрес API", ["Адрес API", "API address", "Endereço da API", "نشانی API", "Dirección de API", "Adresse de l’API", "Indirizzo API", "API 地址"]),
    ("Путь к lms.exe (пусто — автообнаружение)", ["Путь к lms.exe (пусто — автообнаружение)", "Path to lms.exe (leave empty to detect automatically)", "Caminho do lms.exe (vazio para detectar automaticamente)", "مسیر lms.exe (برای شناسایی خودکار خالی بگذارید)", "Ruta de lms.exe (vacío para detección automática)", "Chemin vers lms.exe (vide pour détection automatique)", "Percorso di lms.exe (vuoto per il rilevamento automatico)", "lms.exe 路径（留空以自动检测）"]),
    ("Общие настройки", ["Общие настройки", "General settings", "Configurações gerais", "تنظیمات عمومی", "Ajustes generales", "Paramètres généraux", "Impostazioni generali", "常规设置"]),
    ("Открывать программу", ["Открывать программу", "Open application", "Abrir aplicativo", "باز کردن برنامه", "Abrir la aplicación", "Ouvrir l’application", "Apri applicazione", "打开程序"]),
    ("В трее", ["В трее", "In tray", "Na bandeja", "در سینی", "En la bandeja", "Dans la zone de notification", "Nell’area di notifica", "最小化到托盘"]),
    ("В окне", ["В окне", "In a window", "Em uma janela", "در پنجره", "En una ventana", "Dans une fenêtre", "In una finestra", "在窗口中"]),
    ("Интервал обновления", ["Интервал обновления", "Refresh interval", "Intervalo de atualização", "فاصلهٔ به‌روزرسانی", "Intervalo de actualización", "Intervalle d’actualisation", "Intervallo di aggiornamento", "刷新间隔"]),
    ("Показывать текстовый мониторинг в панели задач", ["Показывать текстовый мониторинг в панели задач", "Show text monitoring in the taskbar", "Mostrar monitoramento de texto na barra de tarefas", "نمایش پایش متنی در نوار وظیفه", "Mostrar monitorización de texto en la barra de tareas", "Afficher le suivi textuel dans la barre des tâches", "Mostra monitoraggio testuale nella barra delle applicazioni", "在任务栏显示文本监控"]),
    ("Запускать вместе с Windows", ["Запускать вместе с Windows", "Start with Windows", "Iniciar com o Windows", "اجرای خودکار با ویندوز", "Iniciar con Windows", "Lancer avec Windows", "Avvia con Windows", "随 Windows 启动"]),
    ("Настройки сохраняются рядом с EXE и переживают обновление Portable.", ["Настройки сохраняются рядом с EXE и переживают обновление Portable.", "Settings are saved beside the EXE and preserved during Portable updates.", "As configurações são salvas ao lado do EXE e preservadas nas atualizações do Portable.", "تنظیمات کنار فایل EXE ذخیره می‌شوند و با به‌روزرسانی نسخهٔ قابل‌حمل حفظ می‌شوند.", "Los ajustes se guardan junto al EXE y se conservan al actualizar Portable.", "Les paramètres sont enregistrés à côté de l’EXE et conservés lors des mises à jour Portable.", "Le impostazioni vengono salvate accanto all’EXE e mantenute negli aggiornamenti Portable.", "设置保存在 EXE 旁边，便携版更新时会保留。"]),
    ("Оформление", ["Оформление", "Appearance", "Aparência", "ظاهر", "Apariencia", "Apparence", "Aspetto", "外观"]),
    ("Язык", ["Язык", "Language", "Idioma", "زبان", "Idioma", "Langue", "Lingua", "语言"]),
    ("Тема приложения", ["Тема приложения", "App theme", "Tema do aplicativo", "تم برنامه", "Tema de la aplicación", "Thème de l’application", "Tema dell’applicazione", "应用主题"]),
    ("Как в системе", ["Как в системе", "System", "Como no sistema", "مطابق سیستم", "Como el sistema", "Comme le système", "Come nel sistema", "跟随系统"]),
    ("Светлая", ["Светлая", "Light", "Claro", "روشن", "Claro", "Clair", "Chiaro", "浅色"]),
    ("Тёмная", ["Тёмная", "Dark", "Escuro", "تیره", "Oscuro", "Sombre", "Scuro", "深色"]),
    ("Доступна новая версия {}. Скачать и установить её?", ["Доступна новая версия {}. Скачать и установить её?", "Version {} is available. Download and install it?", "A versão {} está disponível. Baixar e instalar?", "نسخهٔ {} در دسترس است. دانلود و نصب شود؟", "La versión {} está disponible. ¿Descargarla e instalarla?", "La version {} est disponible. La télécharger et l’installer ?", "È disponibile la versione {}. Scaricarla e installarla?", "新版本 {} 已发布。要下载并安装吗？"]),
    ("Обновление M4 PC Monitor", ["Обновление M4 PC Monitor", "M4 PC Monitor update", "Atualização do M4 PC Monitor", "به‌روزرسانی M4 PC Monitor", "Actualización de M4 PC Monitor", "Mise à jour de M4 PC Monitor", "Aggiornamento M4 PC Monitor", "M4 PC Monitor 更新"]),
    ("Ошибка обновления", ["Ошибка обновления", "Update error", "Erro de atualização", "خطای به‌روزرسانی", "Error de actualización", "Erreur de mise à jour", "Errore di aggiornamento", "更新错误"]),
    ("Проверка SHA-256 установщика не пройдена", ["Проверка SHA-256 установщика не пройдена", "Installer SHA-256 verification failed", "Falha na verificação SHA-256 do instalador", "تأیید SHA-256 نصب‌کننده ناموفق بود", "Falló la verificación SHA-256 del instalador", "Échec de la vérification SHA-256 de l’installateur", "Verifica SHA-256 del programma di installazione non riuscita", "安装程序 SHA-256 校验失败"]),
    ("Файл установщика повреждён или имеет неверный формат", ["Файл установщика повреждён или имеет неверный формат", "The installer file is damaged or has an invalid format", "O arquivo do instalador está danificado ou em formato inválido", "فایل نصب‌کننده خراب است یا قالب معتبری ندارد", "El instalador está dañado o tiene un formato no válido", "Le fichier d’installation est endommagé ou son format est invalide", "Il file di installazione è danneggiato o ha un formato non valido", "安装程序文件已损坏或格式无效"]),
    ("Не удалось запустить установщик", ["Не удалось запустить установщик", "Could not start the installer", "Não foi possível iniciar o instalador", "اجرای نصب‌کننده ممکن نشد", "No se pudo iniciar el instalador", "Impossible de démarrer l’installateur", "Impossibile avviare il programma di installazione", "无法启动安装程序"]),
    ("Обновления", ["Обновления", "Updates", "Atualizações", "به‌روزرسانی‌ها", "Actualizaciones", "Mises à jour", "Aggiornamenti", "更新"]),
    ("Проверить обновления", ["Проверить обновления", "Check for updates", "Verificar atualizações", "بررسی به‌روزرسانی‌ها", "Buscar actualizaciones", "Rechercher des mises à jour", "Controlla aggiornamenti", "检查更新"]),
    ("Текущая версия: {}", ["Текущая версия: {}", "Current version: {}", "Versão atual: {}", "نسخهٔ فعلی: {}", "Versión actual: {}", "Version actuelle : {}", "Versione corrente: {}", "当前版本：{}"]),
    ("Нажмите кнопку для проверки GitHub", ["Нажмите кнопку для проверки GitHub", "Click to check GitHub for updates", "Clique para verificar atualizações no GitHub", "برای بررسی به‌روزرسانی‌ها روی دکمه بزنید", "Haz clic para buscar actualizaciones en GitHub", "Cliquez pour rechercher les mises à jour sur GitHub", "Fai clic per cercare aggiornamenti su GitHub", "点击按钮以检查 GitHub 更新"]),
    ("Проверка обновлений…", ["Проверка обновлений…", "Checking for updates…", "Verificando atualizações…", "در حال بررسی به‌روزرسانی‌ها…", "Buscando actualizaciones…", "Recherche de mises à jour…", "Ricerca aggiornamenti…", "正在检查更新…"]),
    ("Доступна версия {}", ["Доступна версия {}", "Version {} is available", "A versão {} está disponível", "نسخهٔ {} در دسترس است", "La versión {} está disponible", "La version {} est disponible", "È disponibile la versione {}", "有可用版本 {}"]),
    ("Установлена последняя версия", ["Установлена последняя версия", "You have the latest version", "Você tem a versão mais recente", "آخرین نسخه نصب شده است", "Tienes la última versión", "La dernière version est installée", "È installata l’ultima versione", "当前已是最新版本"]),
    ("Скачивается версия {}…", ["Скачивается версия {}…", "Downloading version {}…", "Baixando a versão {}…", "در حال دریافت نسخهٔ {}…", "Descargando la versión {}…", "Téléchargement de la version {}…", "Download della versione {}…", "正在下载版本 {}…"]),
    ("Не удалось проверить обновления", ["Не удалось проверить обновления", "Could not check for updates", "Não foi possível verificar atualizações", "بررسی به‌روزرسانی‌ها ممکن نشد", "No se pudieron buscar actualizaciones", "Impossible de vérifier les mises à jour", "Impossibile verificare gli aggiornamenti", "检查更新失败"]),
    ("Шрифт", ["Шрифт", "Font", "Fonte", "فونت", "Fuente", "Police", "Carattere", "字体"]),
    ("Добавить шрифт…", ["Добавить шрифт…", "Add font…", "Adicionar fonte…", "افزودن فونت…", "Añadir fuente…", "Ajouter une police…", "Aggiungi font…", "添加字体…"]),
    ("Размер текста", ["Размер текста", "Text size", "Tamanho do texto", "اندازهٔ متن", "Tamaño del texto", "Taille du texte", "Dimensione testo", "文字大小"]),
    ("Не выбрано", ["Не выбрано", "None selected", "Nenhum selecionado", "هیچ‌کدام", "Ninguno seleccionado", "Aucun sélectionné", "Nessuna selezione", "未选择"]),
    ("Загрузка", ["Загрузка", "Loading", "Carregando", "در حال بارگذاری", "Cargando", "Chargement", "Caricamento", "加载中"]),
    ("Подготовка", ["Подготовка", "Preparing", "Preparando", "در حال آماده‌سازی", "Preparando", "Préparation", "Preparazione", "准备中"]),
    ("Генерация", ["Генерация", "Generating", "Gerando", "در حال تولید", "Generando", "Génération", "Generazione", "生成中"]),
    ("Выгрузка", ["Выгрузка", "Unloading", "Descarregando", "در حال تخلیه", "Descargando", "Déchargement", "Scaricamento", "卸载中"]),
    ("Ожидание", ["Ожидание", "Idle", "Inativo", "آماده", "En espera", "Inactif", "Inattivo", "空闲"]),
    ("Статус неизвестен", ["Статус неизвестен", "Unknown status", "Status desconhecido", "وضعیت نامشخص", "Estado desconocido", "État inconnu", "Stato sconosciuto", "状态未知"]),
    ("Нет загруженной модели", ["Нет загруженной модели", "No model loaded", "Nenhum modelo carregado", "مدلی بارگذاری نشده", "No hay ningún modelo cargado", "Aucun modèle chargé", "Nessun modello caricato", "未加载模型"]),
    ("Провайдер датчиков", ["Провайдер датчиков", "Sensor provider", "Provedor de sensores", "ارائه‌دهنده حسگر", "Proveedor de sensores", "Fournisseur de capteurs", "Provider sensori", "传感器提供程序"]),
    ("Ошибка сохранения", ["Ошибка сохранения", "Save error", "Erro ao salvar", "خطا در ذخیره", "Error al guardar", "Erreur d’enregistrement", "Errore di salvataggio", "保存错误"]),
    ("Автозапуск включён", ["Автозапуск включён", "Autostart enabled", "Inicialização automática ativada", "اجرای خودکار فعال شد", "Inicio automático activado", "Démarrage automatique activé", "Avvio automatico attivato", "已启用开机启动"]),
    ("Автозапуск выключен", ["Автозапуск выключен", "Autostart disabled", "Inicialização automática desativada", "اجرای خودکار غیرفعال شد", "Inicio automático desactivado", "Démarrage automatique désactivé", "Avvio automatico disattivato", "已禁用开机启动"]),
    ("Сеть ↓", ["Сеть ↓", "Network ↓", "Rede ↓", "شبکه ↓", "Red ↓", "Réseau ↓", "Rete ↓", "网络 ↓"]),
    ("Сеть ↑", ["Сеть ↑", "Network ↑", "Rede ↑", "شبکه ↑", "Red ↑", "Réseau ↑", "Rete ↑", "网络 ↑"]),
    ("Открыть", ["Открыть", "Open", "Abrir", "باز کردن", "Abrir", "Ouvrir", "Apri", "打开"]),
    ("Выход", ["Выход", "Exit", "Sair", "خروج", "Salir", "Quitter", "Esci", "退出"]),
    ("Не удалось изменить автозапуск", ["Не удалось изменить автозапуск", "Could not change autostart", "Não foi possível alterar a inicialização automática", "تغییر اجرای خودکار ممکن نشد", "No se pudo cambiar el inicio automático", "Impossible de modifier le démarrage automatique", "Impossibile modificare l’avvio automatico", "无法更改开机启动设置"]),
    ("Шрифт {} добавлен в Portable и выбран", ["Шрифт {} добавлен в Portable и выбран", "Font {} added to Portable and selected", "Fonte {} adicionada ao Portable e selecionada", "فونت {} به نسخهٔ قابل‌حمل افزوده و انتخاب شد", "Fuente {} añadida a Portable y seleccionada", "Police {} ajoutée à Portable et sélectionnée", "Font {} aggiunto a Portable e selezionato", "字体 {} 已添加到便携版并设为当前字体"]),
    ("Не удалось добавить шрифт", ["Не удалось добавить шрифт", "Could not add font", "Não foi possível adicionar a fonte", "افزودن فونت ممکن نشد", "No se pudo añadir la fuente", "Impossible d’ajouter la police", "Impossibile aggiungere il font", "无法添加字体"]),
    ("Не удалось открыть выбор шрифта", ["Не удалось открыть выбор шрифта", "Could not open font picker", "Não foi possível abrir o seletor de fontes", "باز کردن انتخاب‌گر فونت ممکن نشد", "No se pudo abrir el selector de fuentes", "Impossible d’ouvrir le sélecteur de polices", "Impossibile aprire il selettore dei font", "无法打开字体选择器"]),
    ("Добавить шрифт в M4 PC Monitor", ["Добавить шрифт в M4 PC Monitor", "Add a font to M4 PC Monitor", "Adicionar fonte ao M4 PC Monitor", "افزودن فونت به M4 PC Monitor", "Añadir fuente a M4 PC Monitor", "Ajouter une police à M4 PC Monitor", "Aggiungi un font a M4 PC Monitor", "向 M4 PC Monitor 添加字体"]),
    ("Prompt processing", ["Обработка запроса", "Prompt processing", "Processando prompt", "پردازش ورودی", "Procesando prompt", "Traitement du prompt", "Elaborazione prompt", "提示词处理中"]),
    ("LM Studio offline", ["LM Studio не в сети", "LM Studio offline", "LM Studio offline", "LM Studio آفلاین است", "LM Studio sin conexión", "LM Studio hors ligne", "LM Studio non in linea", "LM Studio 离线"]),
];

pub fn tr(language: Language, source: &'static str) -> &'static str {
    TEXT.iter()
        .find(|(key, _)| *key == source)
        .map(|(_, translations)| translations[language.index()])
        .unwrap_or(source)
}

pub fn tr_format(language: Language, source: &'static str, args: &[String]) -> String {
    let mut output = String::new();
    let mut rest = tr(language, source);
    let mut index = 0;
    while let Some(open) = rest.find('{') {
        output.push_str(&rest[..open]);
        let Some(relative_close) = rest[open..].find('}') else {
            output.push_str(&rest[open..]);
            return output;
        };
        let close = open + relative_close;
        output.push_str(
            args.get(index)
                .map(String::as_str)
                .unwrap_or(&rest[open..=close]),
        );
        index += 1;
        rest = &rest[close + 1..];
    }
    output.push_str(rest);
    output
}

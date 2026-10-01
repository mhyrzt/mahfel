import type { Locale } from "./config";

export interface Messages {
  language: Record<Locale, string>;
  recordingExports: {
    title: string;
    empty: string;
    loading: string;
    unavailable: string;
    download: string;
    retry: string;
    retryPlayback: string;
    playbackFailed: string;
    queued: string;
    rendering: string;
    ready: string;
    failed: string;
    preparing: string;
    partial: string;
    permission: string;
    retryFailed: string;
    play: string;
    remove: string;
    removeTitle: string;
    removeDescription: string;
    removeCancel: string;
    removeFailed: string;
  };
  recordingPlayback: {
    open: string;
    title: string;
    loading: string;
    unavailable: string;
    play: string;
    pause: string;
    seek: string;
    position: string;
    audio: string;
  };
  appearance: {
    label: string;
    light: string;
    dark: string;
    system: string;
  };
  account: {
    title: string;
    description: string;
    openSettings: string;
    preferences: string;
    profile: string;
    picture: string;
    name: string;
    email: string;
    notSet: string;
    calendarType: string;
    gregorian: string;
    persian: string;
    language: string;
    inputs: string;
    microphone: string;
    camera: string;
    enablePreview: string;
    disablePreview: string;
    previewOn: string;
    previewOff: string;
    previewDescription: string;
    previewDenied: string;
    noMicrophones: string;
    noCameras: string;
    chooseMicrophone: string;
    chooseCamera: string;
    microphoneLevel: string;
  };
  shell: {
    workspaceNavigation: string;
    mobileNavigation: string;
    account: string;
    rooms: string;
    sessions: string;
    create: string;
    createRoom: string;
    scheduleSession: string;
    createMenuTitle: string;
    createMenuDescription: string;
    logout: string;
    logoutFailed: string;
  };
  home: {
    eyebrow: string;
    title: string;
    description: string;
    signIn: string;
    supportingText: string;
    status: string;
    footer: string;
  };
  sessions: {
    sessionTitle: string;
    participants: string;
    admissionTab: string;
    roles: Record<"manager" | "moderator" | "member", string>;
    participantActionsLabel: string;
    emptyRoster: string;
    muted: string;
    microphoneOn: string;
    speaking: string;
    cameraOff: string;
    cameraActive: string;
    screenSharePreview: string;
    waitingTitle: string;
    waitingDescription: string;
    waitingHint: string;
    checkAdmission: string;
    admissionQueue: string;
    noAdmissionRequests: string;
    admit: string;
    reject: string;
    scheduled: string;
    liveNow: string;
    ended: string;
    scheduledFor: string;
    schedulingDetailsMissing: string;
    endedDescription: string;
    liveSession: string;
    inProgress: string;
    stageLabel: string;
    muteMicrophone: string;
    unmuteMicrophone: string;
    turnCameraOff: string;
    turnCameraOn: string;
    shareScreen: string;
    leaveSession: string;
    muteParticipant: string;
    removeParticipant: string;
    blockParticipant: string;
    participationSummary: string;
    started: string;
    duration: string;
    sessionTimingUnavailable: string;
    underMinute: string;
    loadingTitle: string;
    loadingMessage: string;
    rejectedTitle: string;
    unavailableTitle: string;
    retryMessage: string;
    signInAgainMessage: string;
    reconnectingMessage: string;
    connectionUnavailableMessage: string;
    accessBlockedMessage: string;
  };
  collaboration: {
    title: string;
    notes: string;
    whiteboard: string;
    presentation: string;
    permissions: string;
    accessTalk: string;
    accessCamera: string;
    accessScreen: string;
    accessPresent: string;
    accessEdit: string;
    accessManaged: string;
    accessSelf: string;
    loading: string;
    unavailable: string;
    viewOnly: string;
    canEdit: string;
    allowEdit: string;
    revokeEdit: string;
    selectFiles: string;
    present: string;
    stop: string;
    previous: string;
    next: string;
    noPresentation: string;
    connected: string;
    notesEditor: string;
    notesPlaceholder: string;
    noteFormatting: string;
    heading: string;
    paragraph: string;
    bold: string;
    italic: string;
    underline: string;
    undo: string;
    redo: string;
    bullets: string;
    insertImage: string;
    imageFailed: string;
    numbers: string;
    pdfUnavailable: string;
    watchToStart: string;
    mute: string;
    unmute: string;
    playbackFailed: string;
  };
}

const messages: Record<Locale, Messages> = {
  en: {
    language: { en: "English", fa: "Persian" },
    recordingExports: {
      title: "Recordings",
      empty: "No recordings yet.",
      loading: "Loading recordings…",
      unavailable: "Recordings could not be loaded. Try again.",
      download: "Download MP4",
      retry: "Retry export",
      retryPlayback: "Retry replay preparation",
      playbackFailed: "Replay preparation failed",
      queued: "Video export queued",
      rendering: "Creating video",
      ready: "MP4 ready",
      failed: "Video export failed",
      preparing: "Preparing original recording",
      partial: "Some parts of this recording are missing.",
      permission: "Recording downloads are available to room members.",
      retryFailed: "The export could not be retried. Try again.",
      play: "Play recording",
      remove: "Delete recording",
      removeTitle: "Delete this recording?",
      removeDescription:
        "The original replay bundle and any generated video are permanently removed. This cannot be undone.",
      removeCancel: "Keep recording",
      removeFailed: "The recording could not be deleted. Try again.",
    },
    recordingPlayback: {
      open: "Play recording",
      title: "Recording playback",
      loading: "Loading recording…",
      unavailable: "This recording could not be opened. Try again.",
      play: "Play",
      pause: "Pause",
      seek: "Playback position",
      position: "Playback time",
      audio: "Audio tracks",
    },
    appearance: { label: "Appearance", light: "Light", dark: "Dark", system: "System" },
    account: {
      title: "Account",
      description: "Manage your profile and Mahfel preferences.",
      openSettings: "Open account settings",
      preferences: "Preferences",
      profile: "Profile details",
      picture: "Profile picture",
      name: "Name",
      email: "Email",
      notSet: "Not provided",
      calendarType: "Calendar type",
      gregorian: "Gregorian",
      persian: "Persian",
      language: "App language",
      inputs: "Input sources",
      microphone: "Microphone",
      camera: "Camera",
      enablePreview: "Enable device preview",
      disablePreview: "Stop device preview",
      previewOn: "Preview on",
      previewOff: "Preview off",
      previewDescription: "Allow camera and microphone access to preview your selected devices.",
      previewDenied: "Device access is unavailable. Check browser permissions and try again.",
      noMicrophones: "No microphones found",
      noCameras: "No cameras found",
      chooseMicrophone: "Choose a microphone",
      chooseCamera: "Choose a camera",
      microphoneLevel: "Microphone input level",
    },
    shell: {
      workspaceNavigation: "Workspace navigation",
      mobileNavigation: "Mobile workspace navigation",
      account: "Account",
      rooms: "Rooms",
      sessions: "Sessions",
      create: "Create",
      createRoom: "Create room",
      scheduleSession: "Schedule session",
      createMenuTitle: "Create something new",
      createMenuDescription: "Choose what you would like to add to your workspace.",
      logout: "Log out",
      logoutFailed: "Could not log out. Please try again.",
    },
    home: {
      eyebrow: "A place to meet",
      title: "Good conversations start here.",
      description:
        "Meet, share ideas, and work together in a space that keeps people and conversation at the center.",
      signIn: "Sign in to Mahfel",
      supportingText: "A little more room for the conversations that matter.",
      status: "Ready when you are",
      footer: "Thoughtfully made for people, in every language.",
    },
    sessions: {
      sessionTitle: "Session",
      participants: "Participants",
      admissionTab: "Admission",
      roles: { manager: "Manager", moderator: "Moderator", member: "Member" },
      participantActionsLabel: "Actions for {name}",
      emptyRoster: "No one has joined this session yet.",
      muted: "Muted",
      microphoneOn: "Microphone on",
      speaking: "Speaking",
      cameraOff: "Video off",
      cameraActive: "Camera is on",
      screenSharePreview: "Screen share preview",
      waitingTitle: "Waiting for admission",
      waitingDescription:
        "Your request to join {roomName} is pending. Media controls will be available after approval.",
      waitingHint: "Keep this page open while a manager reviews your request.",
      checkAdmission: "Check admission",
      admissionQueue: "Admission queue",
      noAdmissionRequests: "No one is waiting for admission.",
      admit: "Admit {name}",
      reject: "Reject {name}",
      scheduled: "Scheduled",
      liveNow: "Live now",
      ended: "Ended",
      scheduledFor: "Scheduled for {time}",
      schedulingDetailsMissing: "Scheduling details are not set.",
      endedDescription: "This session has ended.",
      liveSession: "Live session",
      inProgress: "In progress",
      stageLabel: "{title} session stage",
      muteMicrophone: "Mute microphone",
      unmuteMicrophone: "Unmute microphone",
      turnCameraOff: "Turn camera off",
      turnCameraOn: "Turn camera on",
      shareScreen: "Share screen",
      leaveSession: "Leave session",
      muteParticipant: "Mute participant",
      removeParticipant: "Remove from session",
      blockParticipant: "Block from room",
      participationSummary: "Participation summary",
      started: "Started",
      duration: "Duration",
      sessionTimingUnavailable: "Session timing is unavailable.",
      underMinute: "Less than a minute",
      loadingTitle: "Connecting to session",
      loadingMessage: "Loading session details.",
      rejectedTitle: "Admission was not approved",
      unavailableTitle: "Session unavailable",
      retryMessage: "Please try again.",
      signInAgainMessage: "Your sign-in expired. Sign in again to continue.",
      reconnectingMessage: "Reconnecting to the session…",
      connectionUnavailableMessage: "The session connection is unavailable. Please retry.",
      accessBlockedMessage: "Access to this session was blocked.",
    },
    collaboration: {
      title: "Shared workspace",
      notes: "Notes",
      whiteboard: "Whiteboard",
      presentation: "Presentation",
      permissions: "Access",
      accessTalk: "Talk",
      accessCamera: "Camera",
      accessScreen: "Share screen",
      accessPresent: "Present",
      accessEdit: "Edit",
      accessManaged: "Manager access",
      accessSelf: "Your access",
      loading: "Loading shared workspace…",
      unavailable: "Shared workspace is unavailable.",
      viewOnly: "View only",
      canEdit: "Can edit",
      allowEdit: "Allow editing",
      revokeEdit: "Revoke editing",
      selectFiles: "Choose a PDF, image, or video from Files to present.",
      present: "Present",
      stop: "Stop presenting",
      previous: "Previous",
      next: "Next",
      noPresentation: "Nothing is being presented.",
      connected: "Connected",
      notesEditor: "Collaborative notes",
      notesPlaceholder: "Write notes together",
      noteFormatting: "Note formatting",
      heading: "Heading",
      paragraph: "Paragraph",
      bold: "Bold",
      italic: "Italic",
      underline: "Underline",
      undo: "Undo",
      redo: "Redo",
      bullets: "Bullets",
      insertImage: "Insert image",
      imageFailed: "Image could not be added.",
      numbers: "Numbers",
      pdfUnavailable: "PDF preview unavailable.",
      watchToStart: "Tap to watch together",
      mute: "Mute",
      unmute: "Unmute",
      playbackFailed: "Playback could not be synced.",
    },
  },
  fa: {
    language: { en: "English", fa: "پارسی" },
    recordingExports: {
      title: "ضبط‌ها",
      empty: "هنوز ضبطی وجود ندارد.",
      loading: "در حال بارگذاری ضبط‌ها…",
      unavailable: "بارگذاری ضبط‌ها ممکن نشد. دوباره تلاش کنید.",
      download: "دریافت MP4",
      retry: "تلاش دوباره برای خروجی",
      retryPlayback: "تلاش دوباره برای آماده‌سازی بازپخش",
      playbackFailed: "آماده‌سازی بازپخش ناموفق بود",
      queued: "خروجی ویدیو در صف است",
      rendering: "در حال ساخت ویدیو",
      ready: "ویدیو آماده است",
      failed: "ساخت ویدیو ناموفق بود",
      preparing: "در حال آماده‌سازی ضبط اصلی",
      partial: "بخش‌هایی از این ضبط موجود نیست.",
      permission: "دریافت ضبط برای اعضای اتاق در دسترس است.",
      retryFailed: "شروع دوباره ساخت ویدیو ممکن نشد. دوباره تلاش کنید.",
      play: "پخش ضبط",
      remove: "حذف ضبط",
      removeTitle: "این ضبط حذف شود؟",
      removeDescription:
        "بستهٔ بازپخش اصلی و هر ویدیوی ساخته‌شده برای همیشه حذف می‌شوند. این کار بازگشت‌پذیر نیست.",
      removeCancel: "نگه‌داشتن ضبط",
      removeFailed: "حذف ضبط ممکن نشد. دوباره تلاش کنید.",
    },
    recordingPlayback: {
      open: "پخش ضبط",
      title: "پخش ضبط",
      loading: "در حال بارگذاری ضبط…",
      unavailable: "باز کردن این ضبط ممکن نشد. دوباره تلاش کنید.",
      play: "پخش",
      pause: "مکث",
      seek: "موقعیت پخش",
      position: "زمان پخش",
      audio: "صداهای ضبط‌شده",
    },
    appearance: { label: "پوسته", light: "روشن", dark: "تیره", system: "سیستم" },
    account: {
      title: "حساب کاربری",
      description: "نمایه و تنظیمات محفل را مدیریت کنید.",
      openSettings: "باز کردن تنظیمات حساب کاربری",
      preferences: "ترجیحات",
      profile: "جزئیات نمایه",
      picture: "تصویر نمایه",
      name: "نام",
      email: "رایانامه",
      notSet: "ثبت نشده",
      calendarType: "نوع تقویم",
      gregorian: "میلادی",
      persian: "خورشیدی",
      language: "زبان برنامه",
      inputs: "ورودی‌های رسانه",
      microphone: "میکروفون",
      camera: "دوربین",
      enablePreview: "فعال‌سازی پیش‌نمایش دستگاه‌ها",
      disablePreview: "توقف پیش‌نمایش دستگاه‌ها",
      previewOn: "پیش‌نمایش روشن",
      previewOff: "پیش‌نمایش خاموش",
      previewDescription: "برای پیش‌نمایش دستگاه‌ها، دسترسی دوربین و میکروفون را بدهید.",
      previewDenied: "دسترسی به دستگاه ممکن نیست. مجوز مرورگر را بررسی و دوباره تلاش کنید.",
      noMicrophones: "میکروفونی پیدا نشد",
      noCameras: "دوربینی پیدا نشد",
      chooseMicrophone: "انتخاب میکروفون",
      chooseCamera: "انتخاب دوربین",
      microphoneLevel: "سطح ورودی میکروفون",
    },
    shell: {
      workspaceNavigation: "ناوبری فضای کاری",
      mobileNavigation: "ناوبری فضای کاری در موبایل",
      account: "حساب",
      rooms: "محافل",
      sessions: "دیدارها",
      create: "ایجاد",
      createRoom: "ایجاد محفل",
      scheduleSession: "برنامه‌ریزی دیدار",
      createMenuTitle: "ایجاد مورد جدید",
      createMenuDescription: "انتخاب کنید چه چیزی به فضای کاری اضافه شود.",
      logout: "خروج",
      logoutFailed: "خروج انجام نشد. دوباره تلاش کنید.",
    },
    home: {
      eyebrow: "جایی برای دیدار",
      title: "گفت‌وگوهای خوب از اینجا شروع می‌شوند.",
      description:
        "دور هم جمع شوید، ایده‌ها را به اشتراک بگذارید و با تمرکز بر آدم‌ها و گفت‌وگوها همکاری کنید.",
      signIn: "ورود به محفل",
      supportingText: "فضایی بیشتر برای گفت‌وگوهایی که اهمیت دارند.",
      status: "هر وقت آماده‌اید",
      footer: "با دقت برای آدم‌ها و به هر زبان ساخته شده است.",
    },
    sessions: {
      sessionTitle: "دیدار",
      participants: "شرکت‌کنندگان",
      admissionTab: "پذیرش",
      roles: { manager: "مدیر", moderator: "ناظر", member: "عضو" },
      participantActionsLabel: "عملیات برای {name}",
      emptyRoster: "هنوز کسی به این دیدار نپیوسته است.",
      muted: "بی‌صدا",
      microphoneOn: "میکروفون روشن",
      speaking: "در حال صحبت",
      cameraOff: "دوربین خاموش",
      cameraActive: "دوربین روشن است",
      screenSharePreview: "پیش‌نمایش اشتراک‌گذاری صفحه",
      waitingTitle: "در انتظار پذیرش",
      waitingDescription:
        "درخواست شما برای پیوستن به {roomName} در انتظار بررسی است. پس از تأیید، کنترل‌های رسانه در دسترس خواهند بود.",
      waitingHint: "تا زمان بررسی درخواست توسط مدیر، این صفحه را باز نگه دارید.",
      checkAdmission: "بررسی پذیرش",
      admissionQueue: "صف پذیرش",
      noAdmissionRequests: "درخواستی برای پذیرش وجود ندارد.",
      admit: "پذیرش {name}",
      reject: "رد {name}",
      scheduled: "برنامه‌ریزی‌شده",
      liveNow: "در حال برگزاری",
      ended: "پایان‌یافته",
      scheduledFor: "زمان برگزاری: {time}",
      schedulingDetailsMissing: "جزئیات زمان‌بندی ثبت نشده است.",
      endedDescription: "این دیدار پایان یافته است.",
      liveSession: "دیدار در حال برگزاری",
      inProgress: "در حال برگزاری",
      stageLabel: "صحنه دیدار {title}",
      muteMicrophone: "قطع صدای میکروفون",
      unmuteMicrophone: "روشن کردن میکروفون",
      turnCameraOff: "خاموش کردن دوربین",
      turnCameraOn: "روشن کردن دوربین",
      shareScreen: "اشتراک‌گذاری صفحه",
      leaveSession: "ترک دیدار",
      muteParticipant: "قطع صدای شرکت‌کننده",
      removeParticipant: "حذف از دیدار",
      blockParticipant: "مسدود کردن در محفل",
      participationSummary: "خلاصه مشارکت",
      started: "شروع",
      duration: "مدت",
      sessionTimingUnavailable: "زمان دیدار در دسترس نیست.",
      underMinute: "کمتر از یک دقیقه",
      loadingTitle: "در حال اتصال به دیدار",
      loadingMessage: "جزئیات دیدار در حال بارگذاری است.",
      rejectedTitle: "درخواست پذیرش تأیید نشد",
      unavailableTitle: "دیدار در دسترس نیست",
      retryMessage: "لطفاً دوباره تلاش کنید.",
      signInAgainMessage: "مهلت ورود شما پایان یافته است. برای ادامه دوباره وارد شوید.",
      reconnectingMessage: "در حال اتصال دوباره به دیدار…",
      connectionUnavailableMessage: "اتصال دیدار در دسترس نیست. دوباره تلاش کنید.",
      accessBlockedMessage: "دسترسی به این دیدار مسدود شده است.",
    },
    collaboration: {
      title: "فضای کار مشترک",
      notes: "یادداشت‌ها",
      whiteboard: "تخته سفید",
      presentation: "ارائه",
      permissions: "دسترسی",
      accessTalk: "گفت‌وگو",
      accessCamera: "دوربین",
      accessScreen: "اشتراک صفحه",
      accessPresent: "ارائه",
      accessEdit: "ویرایش",
      accessManaged: "دسترسی مدیر",
      accessSelf: "دسترسی شما",
      loading: "فضای کار مشترک در حال بارگذاری است…",
      unavailable: "فضای کار مشترک در دسترس نیست.",
      viewOnly: "فقط نمایش",
      canEdit: "مجاز به ویرایش",
      allowEdit: "اجازه ویرایش",
      revokeEdit: "لغو اجازه ویرایش",
      selectFiles: "برای ارائه، یک PDF، تصویر یا ویدیو از فایل‌ها انتخاب کنید.",
      present: "ارائه",
      stop: "توقف ارائه",
      previous: "قبلی",
      next: "بعدی",
      noPresentation: "در حال حاضر ارائه‌ای وجود ندارد.",
      connected: "متصل",
      notesEditor: "یادداشت‌های مشترک",
      notesPlaceholder: "با هم یادداشت بنویسید",
      noteFormatting: "قالب‌بندی یادداشت",
      heading: "عنوان",
      paragraph: "بند",
      bold: "پررنگ",
      italic: "کج",
      underline: "زیرخط",
      undo: "واگرد",
      redo: "ازنو",
      bullets: "فهرست نشانه‌دار",
      insertImage: "افزودن تصویر",
      imageFailed: "افزودن تصویر ممکن نشد.",
      numbers: "فهرست شماره‌دار",
      pdfUnavailable: "پیش‌نمایش PDF در دسترس نیست.",
      watchToStart: "برای تماشای هم‌زمان ضربه بزنید",
      mute: "بی‌صدا",
      unmute: "باصدا",
      playbackFailed: "همگام‌سازی پخش ممکن نشد.",
    },
  },
};

export function messagesFor(locale: Locale): Messages {
  return messages[locale];
}

export function formatMessage(template: string, values: Record<string, string | number>): string {
  return Object.entries(values).reduce(
    (message, [key, value]) => message.replaceAll(`{${key}}`, String(value)),
    template,
  );
}

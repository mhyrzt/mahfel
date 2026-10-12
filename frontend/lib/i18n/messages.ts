import type { Locale } from "./config";

export interface Messages {
  interactions: {
    title: string;
    create: string;
    poll: string;
    quiz: string;
    question: string;
    option: string;
    addOption: string;
    removeOption: string;
    multiple: string;
    anonymous: string;
    anonymousHint: string;
    namedHint: string;
    correctOption: string;
    explanation: string;
    exactSet: string;
    saveDraft: string;
    draft: string;
    open: string;
    closed: string;
    launch: string;
    close: string;
    closeQuizHint: string;
    cancel: string;
    edit: string;
    vote: string;
    submit: string;
    submitted: string;
    waitingResults: string;
    changeVote: string;
    retract: string;
    selectOne: string;
    selectMany: string;
    chooseAnswer: string;
    respondents: string;
    responses: string;
    selected: string;
    correct: string;
    incorrect: string;
    multiPercentage: string;
    viewVoters: string;
    more: string;
    empty: string;
    loading: string;
    failed: string;
    conflict: string;
    invalid: string;
    retry: string;
    newActivity: string;
    raiseHand: string;
    lowerHand: string;
    handRaised: string;
    handQueue: string;
    clearHands: string;
    queuePosition: string;
    emptyHands: string;
    slideshow: string;
    slide: string;
    questionNavigation: string;
    questionPosition: string;
    previousQuestion: string;
    nextQuestion: string;
    reactions: string;
    reactionSent: string;
    reactionThrottled: string;
    like: string;
    dislike: string;
    applause: string;
    love: string;
    laugh: string;
    surprise: string;
    celebrate: string;
    fire: string;
    privacy: string;
    named: string;
    finalAnswer: string;
    activeFirst: string;
  };
  language: Record<Locale, string>;
  sessionConnection: {
    title: string;
    signaling: string;
    media: string;
    quality: string;
    latency: string;
    jitter: string;
    packetLoss: string;
    unavailable: string;
    estimate: string;
    idleHint: string;
    status: Record<
      | "connecting"
      | "connected"
      | "reconnecting"
      | "unauthorized"
      | "unavailable"
      | "closed"
      | "offline",
      string
    >;
    mediaStatus: Record<
      "idle" | "connecting" | "connected" | "disconnected" | "unavailable",
      string
    >;
    qualityLabel: Record<"good" | "fair" | "poor" | "unknown", string>;
  };
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
    brand: string;
    titleLead: string;
    titleAccent: string;
    skipToContent: string;
    signingIn: string;
    signInFailed: string;
    capabilitiesLabel: string;
    sessionTitle: string;
    sessionDescription: string;
    chatTitle: string;
    chatDescription: string;
    ideasTitle: string;
    ideasDescription: string;
    filesTitle: string;
    filesDescription: string;
    eyebrow: string;
    title: string;
    description: string;
    signIn: string;
    supportingText: string;
    status: string;
    partoName: string;
    partoGreeting: string;
    partoTalking: string;
    partoReady: string;
    partoInteract: string;
    partoUnavailable: string;
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
    interactions: {
      title: "Polls & quizzes",
      create: "Create",
      poll: "Poll",
      quiz: "Quiz",
      question: "Question",
      option: "Option {number}",
      addOption: "Add option",
      removeOption: "Remove option {number}",
      multiple: "Allow multiple answers",
      anonymous: "Anonymous voting",
      anonymousHint: "Individual choices are hidden from everyone, including hosts.",
      namedHint: "Your name and choices appear alongside the results.",
      correctOption: "Correct option {number}",
      explanation: "Explanation (optional)",
      exactSet: "Select every correct option and no incorrect options.",
      saveDraft: "Save draft",
      draft: "Draft",
      open: "Open",
      closed: "Closed",
      launch: "Launch",
      close: "Close activity",
      closeQuizHint: "Closing reveals the correct answers and results to everyone.",
      cancel: "Cancel",
      edit: "Edit draft",
      vote: "Vote",
      submit: "Submit answer",
      submitted: "Answer submitted",
      waitingResults: "Results will appear when the host closes the quiz.",
      changeVote: "Change vote",
      retract: "Retract vote",
      selectOne: "Choose one option.",
      selectMany: "Choose one or more options.",
      chooseAnswer: "Choose an answer before submitting.",
      respondents: "{count} respondents",
      responses: "{count} submissions",
      selected: "Your choice",
      correct: "Correct",
      incorrect: "Incorrect",
      multiPercentage: "Percentages are based on respondents and may total more than 100%.",
      viewVoters: "View voters",
      more: "Show more",
      empty: "No polls or quizzes yet.",
      loading: "Loading activities…",
      failed: "Could not update session interactions. Try again.",
      conflict:
        "This changed on another device or the host closed it. Review the latest state and try again.",
      invalid: "Check the question, distinct options and correct answers.",
      retry: "Retry",
      newActivity: "A new poll or quiz is open",
      raiseHand: "Raise hand",
      lowerHand: "Lower hand",
      handRaised: "Hand raised",
      handQueue: "Raised hands",
      clearHands: "Lower all hands",
      queuePosition: "Position {number}",
      emptyHands: "No hands raised.",
      slideshow: "slideshow",
      slide: "slide",
      questionNavigation: "Question navigation",
      questionPosition: "Question {current} of {total}",
      previousQuestion: "Previous",
      nextQuestion: "Next",
      reactions: "Reactions",
      reactionSent: "Reaction sent",
      reactionThrottled: "Wait a moment before reacting again.",
      like: "Like",
      dislike: "Dislike",
      applause: "Applause",
      love: "Love",
      laugh: "Laugh",
      surprise: "Surprised",
      celebrate: "Celebrate",
      fire: "Fire",
      privacy: "Voting privacy",
      named: "Named voting",
      finalAnswer: "Your quiz answer cannot be changed after submission.",
      activeFirst: "Close the active activity before launching another.",
    },
    language: { en: "English", fa: "Persian" },
    sessionConnection: {
      title: "Connection details",
      signaling: "Session connection",
      media: "Audio & video",
      quality: "Estimated quality",
      latency: "Latency",
      jitter: "Jitter",
      packetLoss: "Packet loss",
      unavailable: "Not available",
      estimate:
        "Quality is an estimate from recent media measurements. Latency is the round-trip time to the media server.",
      idleHint: "Media measurements appear when you send or receive audio or video.",
      status: {
        connecting: "Connecting",
        connected: "Connected",
        reconnecting: "Reconnecting",
        unauthorized: "Sign-in expired",
        unavailable: "Unavailable",
        closed: "Disconnected",
        offline: "Offline",
      },
      mediaStatus: {
        idle: "Not active",
        connecting: "Connecting",
        connected: "Connected",
        disconnected: "Media interrupted",
        unavailable: "Measurements unavailable",
      },
      qualityLabel: { good: "Good", fair: "Fair", poor: "Poor", unknown: "Not measured" },
    },
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
      brand: "Mahfel",
      titleLead: "Good conversations",
      titleAccent: "start here.",
      skipToContent: "Skip to content",
      signingIn: "Opening sign-in…",
      signInFailed: "We couldn’t open sign-in. Please try again.",
      capabilitiesLabel: "A space to connect and collaborate",
      sessionTitle: "Meet face to face",
      sessionDescription: "Video, voice, and screen sharing.",
      chatTitle: "Keep the conversation",
      chatDescription: "Room chats that stay with your people.",
      ideasTitle: "Think together",
      ideasDescription: "Shared notes and a space to draw.",
      filesTitle: "Bring it all along",
      filesDescription: "Files and ideas, right where you meet.",
      eyebrow: "A place to meet",
      title: "Good conversations start here.",
      description:
        "Meet, share ideas, and work together in a space that keeps people and conversation at the center.",
      signIn: "Sign in to Mahfel",
      supportingText: "A little more room for the conversations that matter.",
      status: "Ready when you are",
      partoName: "Parto",
      partoGreeting: "Hi, I’m Parto. A little light for your next great conversation.",
      partoTalking: "Saying hello",
      partoReady: "Here for a little company",
      partoInteract: "Parto. Tap or press Enter for a playful hop",
      partoUnavailable: "Parto is taking a rest.",
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
    interactions: {
      title: "نظرسنجی و آزمون",
      create: "ساختن",
      poll: "نظرسنجی",
      quiz: "آزمون",
      question: "پرسش",
      option: "گزینهٔ {number}",
      addOption: "افزودن گزینه",
      removeOption: "حذف گزینهٔ {number}",
      multiple: "انتخاب چند گزینه",
      anonymous: "رأی‌گیری ناشناس",
      anonymousHint: "انتخاب‌های افراد از همه، حتی میزبان، پنهان می‌ماند.",
      namedHint: "نام و انتخاب‌های شما همراه نتایج نمایش داده می‌شود.",
      correctOption: "گزینهٔ درست {number}",
      explanation: "توضیح پاسخ (اختیاری)",
      exactSet: "همهٔ گزینه‌های درست را انتخاب کنید؛ هیچ گزینهٔ نادرستی انتخاب نکنید.",
      saveDraft: "ذخیرهٔ پیش‌نویس",
      draft: "پیش‌نویس",
      open: "باز",
      closed: "بسته",
      launch: "شروع",
      close: "پایان رأی‌گیری",
      closeQuizHint: "با پایان آزمون، پاسخ‌های درست و نتایج برای همه نمایش داده می‌شود.",
      cancel: "انصراف",
      edit: "ویرایش پیش‌نویس",
      vote: "ثبت رأی",
      submit: "ثبت پاسخ",
      submitted: "پاسخ ثبت شد",
      waitingResults: "پس از پایان آزمون توسط میزبان، نتایج نمایش داده می‌شود.",
      changeVote: "تغییر رأی",
      retract: "پس‌گرفتن رأی",
      selectOne: "یک گزینه انتخاب کنید.",
      selectMany: "یک یا چند گزینه انتخاب کنید.",
      chooseAnswer: "پیش از ثبت، یک پاسخ انتخاب کنید.",
      respondents: "{count} پاسخ‌دهنده",
      responses: "{count} پاسخ ثبت‌شده",
      selected: "انتخاب شما",
      correct: "درست",
      incorrect: "نادرست",
      multiPercentage:
        "درصدها بر پایهٔ تعداد پاسخ‌دهندگان است و مجموع آن‌ها ممکن است از ۱۰۰٪ بیشتر باشد.",
      viewVoters: "نمایش رأی‌دهندگان",
      more: "نمایش بیشتر",
      empty: "هنوز نظرسنجی یا آزمونی وجود ندارد.",
      loading: "در حال بارگذاری…",
      failed: "به‌روزرسانی ممکن نشد. دوباره تلاش کنید.",
      conflict:
        "وضعیت در دستگاه دیگری تغییر کرده یا میزبان رأی‌گیری را بسته است. وضعیت تازه را بررسی کنید.",
      invalid: "پرسش، گزینه‌های متفاوت و پاسخ‌های درست را بررسی کنید.",
      retry: "تلاش دوباره",
      newActivity: "نظرسنجی یا آزمون تازه‌ای شروع شد",
      raiseHand: "بالا بردن دست",
      lowerHand: "پایین آوردن دست",
      handRaised: "دست بالا",
      handQueue: "دست‌های بالا",
      clearHands: "پایین آوردن همهٔ دست‌ها",
      queuePosition: "نفر {number}",
      emptyHands: "کسی دستش را بالا نبرده است.",
      slideshow: "نمایش اسلایدی",
      slide: "اسلاید",
      questionNavigation: "جابه‌جایی بین پرسش‌ها",
      questionPosition: "پرسش {current} از {total}",
      previousQuestion: "قبلی",
      nextQuestion: "بعدی",
      reactions: "واکنش‌ها",
      reactionSent: "واکنش فرستاده شد",
      reactionThrottled: "برای واکنش بعدی کمی صبر کنید.",
      like: "پسندیدن",
      dislike: "نپسندیدن",
      applause: "تشویق",
      love: "علاقه",
      laugh: "خنده",
      surprise: "تعجب",
      celebrate: "جشن",
      fire: "عالیه",
      privacy: "حریم خصوصی رأی‌گیری",
      named: "رأی‌گیری با نام",
      finalAnswer: "پس از ثبت، پاسخ آزمون قابل تغییر نیست.",
      activeFirst: "پیش از شروع مورد بعدی، رأی‌گیری فعال را ببندید.",
    },
    language: { en: "English", fa: "پارسی" },
    sessionConnection: {
      title: "جزئیات اتصال",
      signaling: "اتصال نشست",
      media: "صدا و تصویر",
      quality: "کیفیت تقریبی",
      latency: "تأخیر",
      jitter: "نوسان تأخیر",
      packetLoss: "اتلاف بسته",
      unavailable: "در دسترس نیست",
      estimate:
        "کیفیت بر اساس اندازه‌گیری‌های اخیر رسانه تخمین زده می‌شود. تأخیر، زمان رفت‌وبرگشت تا سرور رسانه است.",
      idleHint: "اندازه‌گیری رسانه هنگام ارسال یا دریافت صدا و تصویر نمایش داده می‌شود.",
      status: {
        connecting: "در حال اتصال",
        connected: "متصل",
        reconnecting: "اتصال دوباره",
        unauthorized: "ورود منقضی شده",
        unavailable: "در دسترس نیست",
        closed: "قطع شده",
        offline: "آفلاین",
      },
      mediaStatus: {
        idle: "فعال نیست",
        connecting: "در حال اتصال",
        connected: "متصل",
        disconnected: "ارتباط رسانه قطع شده",
        unavailable: "اندازه‌گیری در دسترس نیست",
      },
      qualityLabel: { good: "خوب", fair: "متوسط", poor: "ضعیف", unknown: "اندازه‌گیری نشده" },
    },
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
      brand: "محفل",
      titleLead: "گفت‌وگوهای خوب",
      titleAccent: "از اینجا شروع می‌شوند.",
      skipToContent: "رفتن به محتوای اصلی",
      signingIn: "در حال باز کردن صفحهٔ ورود…",
      signInFailed: "صفحهٔ ورود باز نشد. دوباره تلاش کنید.",
      capabilitiesLabel: "فضایی برای دیدار و همکاری",
      sessionTitle: "روبه‌روی هم، از هر جا",
      sessionDescription: "تماس تصویری، صوتی و اشتراک صفحه.",
      chatTitle: "گفت‌وگو ادامه دارد",
      chatDescription: "پیام‌های شما، کنار آدم‌های اتاق.",
      ideasTitle: "با هم فکر کنید",
      ideasDescription: "یادداشت مشترک و فضایی برای طراحی.",
      filesTitle: "همه‌چیز کنار هم",
      filesDescription: "فایل‌ها و ایده‌ها، همان‌جا که دیدار می‌کنید.",
      eyebrow: "جایی برای دیدار",
      title: "گفت‌وگوهای خوب از اینجا شروع می‌شوند.",
      description:
        "محفل، فضایی برای گفت‌وگو، هم‌فکری و خلق ایده‌هاست. دور هم جمع شوید، تصویر و صفحه‌تان را به اشتراک بگذارید و با هم پیش بروید.",
      signIn: "ورود به محفل",
      supportingText: "فضایی بیشتر برای گفت‌وگوهایی که اهمیت دارند.",
      status: "هر وقت آماده‌اید",
      partoName: "پرتو",
      partoGreeting: "سلام، من پرتو هستم! یه ذره روشنایی برای گفت‌وگوی بعدی‌تون.",
      partoTalking: "در حال سلام کردن",
      partoReady: "اینجام که همراهتون باشم",
      partoInteract: "پرتو. برای یک پرش کوچک، کلیک کنید یا کلید ورود را بزنید",
      partoUnavailable: "پرتو داره استراحت می‌کنه.",
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

import type { Locale } from "./config";

export interface Messages {
  language: Record<Locale, string>;
  shell: {
    workspaceNavigation: string;
    mobileNavigation: string;
    rooms: string;
    allRooms: string;
    sessions: string;
    upcomingSessions: string;
    todaysSchedule: string;
    create: string;
    createRoom: string;
    scheduleSession: string;
    createMenuTitle: string;
    createMenuDescription: string;
  };
  home: {
    eyebrow: string;
    title: string;
    description: string;
    classroomsComingLater: string;
    api: string;
  };
  sessions: {
    participants: string;
    roles: Record<"manager" | "moderator" | "member", string>;
    participantActionsLabel: string;
    emptyRoster: string;
    muted: string;
    microphoneOn: string;
    cameraOff: string;
    cameraActive: string;
    screenSharePreview: string;
    waitingTitle: string;
    waitingDescription: string;
    waitingHint: string;
    admissionQueue: string;
    noAdmissionRequests: string;
    requested: string;
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
    loadingTitle: string;
    loadingMessage: string;
    rejectedTitle: string;
    unavailableTitle: string;
    retryMessage: string;
  };
}

const messages: Record<Locale, Messages> = {
  en: {
    language: { en: "English", fa: "Persian" },
    shell: {
      workspaceNavigation: "Workspace navigation",
      mobileNavigation: "Mobile workspace navigation",
      rooms: "Rooms",
      allRooms: "All rooms",
      sessions: "Sessions",
      upcomingSessions: "Upcoming sessions",
      todaysSchedule: "Today’s schedule",
      create: "Create",
      createRoom: "Create room",
      scheduleSession: "Schedule session",
      createMenuTitle: "Create something new",
      createMenuDescription: "Choose what you would like to add to your workspace.",
    },
    home: {
      eyebrow: "Foundation ready",
      title: "Mahfel",
      description:
        "An education-focused realtime communication and collaboration platform. Product features will be built on this independently deployable web foundation.",
      classroomsComingLater: "Classrooms coming later",
      api: "API",
    },
    sessions: {
      participants: "Participants",
      roles: { manager: "Manager", moderator: "Moderator", member: "Member" },
      participantActionsLabel: "Actions for {name}",
      emptyRoster: "No one has joined this session yet.",
      muted: "Muted",
      microphoneOn: "Microphone on",
      cameraOff: "Video off",
      cameraActive: "Camera is on",
      screenSharePreview: "Screen share preview",
      waitingTitle: "Waiting for admission",
      waitingDescription:
        "Your request to join {roomName} is pending. Media controls will be available after approval.",
      waitingHint: "Keep this page open while a manager reviews your request.",
      admissionQueue: "Admission queue",
      noAdmissionRequests: "No one is waiting for admission.",
      requested: "Requested {requestedAt}",
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
      loadingTitle: "Connecting to session",
      loadingMessage: "Loading session details.",
      rejectedTitle: "Admission was not approved",
      unavailableTitle: "Session unavailable",
      retryMessage: "Please try again.",
    },
  },
  fa: {
    language: { en: "English", fa: "فارسی" },
    shell: {
      workspaceNavigation: "ناوبری فضای کاری",
      mobileNavigation: "ناوبری فضای کاری در موبایل",
      rooms: "اتاق‌ها",
      allRooms: "همه اتاق‌ها",
      sessions: "جلسه‌ها",
      upcomingSessions: "جلسه‌های آینده",
      todaysSchedule: "برنامه امروز",
      create: "ایجاد",
      createRoom: "ایجاد اتاق",
      scheduleSession: "برنامه‌ریزی جلسه",
      createMenuTitle: "ایجاد مورد جدید",
      createMenuDescription: "انتخاب کنید چه چیزی به فضای کاری اضافه شود.",
    },
    home: {
      eyebrow: "زیرساخت آماده است",
      title: "محفل",
      description:
        "یک پلتفرم ارتباط و همکاری بلادرنگ با تمرکز بر آموزش. قابلیت‌های محصول بر این زیرساخت مستقل وب ساخته می‌شوند.",
      classroomsComingLater: "کلاس‌ها به‌زودی",
      api: "API",
    },
    sessions: {
      participants: "شرکت‌کنندگان",
      roles: { manager: "مدیر", moderator: "ناظر", member: "عضو" },
      participantActionsLabel: "عملیات برای {name}",
      emptyRoster: "هنوز کسی به این جلسه نپیوسته است.",
      muted: "بی‌صدا",
      microphoneOn: "میکروفون روشن",
      cameraOff: "دوربین خاموش",
      cameraActive: "دوربین روشن است",
      screenSharePreview: "پیش‌نمایش اشتراک‌گذاری صفحه",
      waitingTitle: "در انتظار پذیرش",
      waitingDescription:
        "درخواست شما برای پیوستن به {roomName} در انتظار بررسی است. پس از تأیید، کنترل‌های رسانه در دسترس خواهند بود.",
      waitingHint: "تا زمان بررسی درخواست توسط مدیر، این صفحه را باز نگه دارید.",
      admissionQueue: "صف پذیرش",
      noAdmissionRequests: "درخواستی برای پذیرش وجود ندارد.",
      requested: "زمان درخواست: {requestedAt}",
      admit: "پذیرش {name}",
      reject: "رد {name}",
      scheduled: "برنامه‌ریزی‌شده",
      liveNow: "در حال برگزاری",
      ended: "پایان‌یافته",
      scheduledFor: "زمان برگزاری: {time}",
      schedulingDetailsMissing: "جزئیات زمان‌بندی ثبت نشده است.",
      endedDescription: "این جلسه پایان یافته است.",
      liveSession: "جلسه در حال برگزاری",
      inProgress: "در حال برگزاری",
      stageLabel: "صحنه جلسه {title}",
      muteMicrophone: "قطع صدای میکروفون",
      unmuteMicrophone: "روشن کردن میکروفون",
      turnCameraOff: "خاموش کردن دوربین",
      turnCameraOn: "روشن کردن دوربین",
      shareScreen: "اشتراک‌گذاری صفحه",
      leaveSession: "ترک جلسه",
      muteParticipant: "قطع صدای شرکت‌کننده",
      removeParticipant: "حذف از جلسه",
      blockParticipant: "مسدود کردن در اتاق",
      participationSummary: "خلاصه مشارکت",
      started: "شروع",
      duration: "مدت",
      loadingTitle: "در حال اتصال به جلسه",
      loadingMessage: "جزئیات جلسه در حال بارگذاری است.",
      rejectedTitle: "درخواست پذیرش تأیید نشد",
      unavailableTitle: "جلسه در دسترس نیست",
      retryMessage: "لطفاً دوباره تلاش کنید.",
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

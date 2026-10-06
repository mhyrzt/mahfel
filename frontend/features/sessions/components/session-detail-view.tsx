import type { ReactNode } from "react";

export function SessionDetailView({
  breadcrumb,
  lifecycle,
  stage,
  collaboration,
  activities,
  participation,
  files,
  invites,
}: Readonly<{
  breadcrumb?: ReactNode;
  lifecycle?: ReactNode;
  stage?: ReactNode;
  collaboration?: ReactNode;
  activities?: ReactNode;
  participation?: ReactNode;
  files?: ReactNode;
  invites?: ReactNode;
}>) {
  return (
    <div
      className={stage ? "h-full min-h-0" : "mx-auto max-w-5xl space-y-6 pb-16"}
      data-testid="session-detail"
    >
      {breadcrumb}
      {lifecycle}
      {stage}
      {collaboration}
      {activities}
      {participation}
      {files}
      {invites}
    </div>
  );
}

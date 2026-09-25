import type { ReactNode } from "react";

export function SessionDetailView({
  lifecycle,
  stage,
  collaboration,
  participation,
  files,
  invites,
}: Readonly<{
  lifecycle?: ReactNode;
  stage?: ReactNode;
  collaboration?: ReactNode;
  participation?: ReactNode;
  files?: ReactNode;
  invites?: ReactNode;
}>) {
  return (
    <div className="space-y-8" data-testid="session-detail">
      {lifecycle}
      {stage}
      {collaboration}
      {participation}
      {files}
      {invites}
    </div>
  );
}

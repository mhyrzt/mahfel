import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/nextjs";

import { Checkbox } from "../checkbox";
import { DirectionProvider } from "../direction";
import { Field, FieldDescription, FieldError, FieldLabel } from "../field";
import { Input } from "../input";
import { NativeSelect, NativeSelectOption } from "../native-select";
import { Progress } from "../progress";
import { SegmentedControl } from "../segmented-control";
import { Slider } from "../slider";
import { Switch } from "../switch";

function FormControls({
  locale = "en",
  disabled = false,
  invalid = false,
}: Readonly<{
  locale?: "en" | "fa";
  disabled?: boolean;
  invalid?: boolean;
}>) {
  const [enabled, setEnabled] = useState(true);
  const [preference, setPreference] = useState("private");
  const [seek, setSeek] = useState([30]);
  const fa = locale === "fa";
  return (
    <DirectionProvider dir={fa ? "rtl" : "ltr"}>
      <div className="w-80 max-w-full space-y-5 p-4" dir={fa ? "rtl" : "ltr"} lang={locale}>
        <Field data-invalid={invalid}>
          <FieldLabel htmlFor="story-name">{fa ? "نام محفل" : "Room name"}</FieldLabel>
          <Input
            id="story-name"
            disabled={disabled}
            aria-invalid={invalid}
            aria-describedby="story-name-detail"
          />
          {invalid ? (
            <FieldError id="story-name-detail">
              {fa ? "نام محفل را وارد کنید." : "Enter a room name."}
            </FieldError>
          ) : (
            <FieldDescription id="story-name-detail">
              {fa ? "برای دعوت مهمانان" : "For inviting guests"}
            </FieldDescription>
          )}
        </Field>
        <Field orientation="horizontal">
          <Switch
            id="story-preview"
            disabled={disabled}
            checked={enabled}
            onCheckedChange={setEnabled}
          />
          <FieldLabel htmlFor="story-preview">{fa ? "پیش‌نمایش" : "Preview"}</FieldLabel>
        </Field>
        <Field orientation="horizontal">
          <Checkbox id="story-record" disabled={disabled} defaultChecked />
          <FieldLabel htmlFor="story-record">
            {fa ? "ضبط خودکار" : "Automatic recording"}
          </FieldLabel>
        </Field>
        <SegmentedControl
          disabled={disabled}
          label={fa ? "دسترسی" : "Visibility"}
          value={preference}
          onValueChange={setPreference}
          options={[
            { value: "private", label: fa ? "خصوصی" : "Private" },
            { value: "public", label: fa ? "عمومی" : "Public" },
          ]}
        />
        <Field>
          <FieldLabel htmlFor="story-role">{fa ? "نقش" : "Role"}</FieldLabel>
          <NativeSelect id="story-role" disabled={disabled}>
            <NativeSelectOption>{fa ? "عضو" : "Member"}</NativeSelectOption>
            <NativeSelectOption>{fa ? "مدیر" : "Manager"}</NativeSelectOption>
          </NativeSelect>
        </Field>
        <Slider
          aria-label={fa ? "موقعیت پخش" : "Playback position"}
          dir="ltr"
          disabled={disabled}
          value={seek}
          onValueChange={setSeek}
        />
        <Progress aria-label={fa ? "پیشرفت خروجی" : "Export progress"} value={35} />
      </div>
    </DirectionProvider>
  );
}

const meta = {
  title: "UI/Form controls",
  component: FormControls,
  parameters: { layout: "centered" },
} satisfies Meta<typeof FormControls>;
export default meta;
type Story = StoryObj<typeof meta>;
export const English: Story = {};
export const Persian: Story = { args: { locale: "fa" } };
export const Disabled: Story = { args: { disabled: true } };
export const Invalid: Story = { args: { invalid: true } };

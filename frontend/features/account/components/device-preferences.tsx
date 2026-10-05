"use client";

import { useEffect, useId, useRef, useState } from "react";
import { Camera, Mic } from "lucide-react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { Switch } from "@/components/ui/switch";
import { Field, FieldLabel } from "@/components/ui/field";

interface DevicePreferencesProps {
  locale: Locale;
}

export function DevicePreferences({ locale }: Readonly<DevicePreferencesProps>) {
  const microphoneFieldId = useId();
  const cameraFieldId = useId();
  const messages = messagesFor(locale).account;
  const [previewEnabled, setPreviewEnabled] = useState(true);
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [microphoneId, setMicrophoneId] = useState("");
  const [cameraId, setCameraId] = useState("");
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [level, setLevel] = useState(0);
  const [error, setError] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  function updateDevices(nextDevices: MediaDeviceInfo[]) {
    setDevices(nextDevices);
    setMicrophoneId(
      (current) =>
        current || nextDevices.find((device) => device.kind === "audioinput")?.deviceId || "",
    );
    setCameraId(
      (current) =>
        current || nextDevices.find((device) => device.kind === "videoinput")?.deviceId || "",
    );
  }

  useEffect(() => {
    if (!navigator.mediaDevices?.enumerateDevices) {
      return;
    }

    void navigator.mediaDevices
      .enumerateDevices()
      .then(updateDevices)
      .catch(() => setError(true));
  }, []);

  useEffect(() => {
    let currentStream: MediaStream | null = null;
    let active = true;
    const stopPreview = () => {
      active = false;
      currentStream?.getTracks().forEach((track) => track.stop());
      setStream(null);
      setLevel(0);
    };

    if (!previewEnabled) {
      return stopPreview;
    }

    const media = navigator.mediaDevices;
    if (!media?.getUserMedia) {
      queueMicrotask(() => {
        if (active) {
          setError(true);
        }
      });
      return stopPreview;
    }

    const audio: MediaTrackConstraints | boolean = microphoneId
      ? { deviceId: { exact: microphoneId } }
      : true;
    const video: MediaTrackConstraints | boolean = cameraId
      ? { deviceId: { exact: cameraId } }
      : true;

    void media
      .getUserMedia({ audio, video })
      .then(async (nextStream) => {
        if (!active) {
          nextStream.getTracks().forEach((track) => track.stop());
          return;
        }
        currentStream = nextStream;
        setStream(nextStream);
        setError(false);
        updateDevices(await media.enumerateDevices());
      })
      .catch(() => setError(true));

    return stopPreview;
  }, [cameraId, microphoneId, previewEnabled]);

  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      video.srcObject = stream;
    }
  }, [stream]);

  useEffect(() => {
    const audioTrack = stream?.getAudioTracks()[0];
    if (!audioTrack) {
      return () => {};
    }

    const context = new AudioContext();
    const source = context.createMediaStreamSource(new MediaStream([audioTrack]));
    const analyzer = context.createAnalyser();
    const samples = new Uint8Array(analyzer.fftSize);
    source.connect(analyzer);
    let frame = 0;
    let lastUpdate = 0;

    const measure = (time: number) => {
      analyzer.getByteTimeDomainData(samples);
      if (time - lastUpdate > 100) {
        const sum = samples.reduce((total, sample) => total + (sample - 128) ** 2, 0);
        setLevel(Math.min(100, Math.round(Math.sqrt(sum / samples.length) * 2.2)));
        lastUpdate = time;
      }
      frame = requestAnimationFrame(measure);
    };
    frame = requestAnimationFrame(measure);

    return () => {
      cancelAnimationFrame(frame);
      source.disconnect();
      analyzer.disconnect();
      void context.close();
    };
  }, [stream]);

  const microphones = devices.filter((device) => device.kind === "audioinput");
  const cameras = devices.filter((device) => device.kind === "videoinput");
  let previewMessage = "";
  if (error) {
    previewMessage = messages.previewDenied;
  }

  return (
    <section className="space-y-4 rounded-xl bg-card p-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="font-medium">{messages.inputs}</h3>
          <p className="mt-1 text-sm text-muted-foreground">{messages.previewDescription}</p>
        </div>
        <div className="mt-1 flex shrink-0 items-center gap-2">
          <span className="text-sm text-muted-foreground" aria-hidden="true">
            {previewEnabled ? messages.previewOn : messages.previewOff}
          </span>
          <Switch
            aria-label={previewEnabled ? messages.disablePreview : messages.enablePreview}
            checked={previewEnabled}
            onCheckedChange={setPreviewEnabled}
          />
        </div>
      </div>
      <div className="grid items-start gap-5 sm:grid-cols-2">
        <Field className="min-w-0 gap-2">
          <FieldLabel className="h-5" htmlFor={microphoneFieldId}>
            <Mic aria-hidden="true" className="size-4 text-muted-foreground" />
            {messages.microphone}
          </FieldLabel>
          <Select
            disabled={microphones.length === 0}
            onValueChange={setMicrophoneId}
            value={microphoneId || undefined}
          >
            <SelectTrigger id={microphoneFieldId} aria-label={messages.chooseMicrophone}>
              <SelectValue placeholder={messages.noMicrophones} />
            </SelectTrigger>
            <SelectContent
              className="max-w-[min(24rem,calc(100vw-2rem))]"
              dir={locale === "fa" ? "rtl" : "ltr"}
            >
              {microphones.map((device, index) => (
                <SelectItem
                  className="[&_[data-radix-select-item-text]]:truncate"
                  key={device.deviceId}
                  value={device.deviceId}
                >
                  {device.label || `${messages.microphone} ${index + 1}`}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <div
            aria-label={messages.microphoneLevel}
            aria-valuemax={100}
            aria-valuemin={0}
            aria-valuenow={level}
            className="h-2 overflow-hidden rounded-full bg-muted"
            dir="ltr"
            role="meter"
          >
            <div
              className="h-full w-full origin-left bg-gradient-to-r from-emerald-500 via-amber-400 to-red-500 transition-transform"
              style={{ transform: `scaleX(${level / 100})` }}
            />
          </div>
        </Field>
        <Field className="min-w-0 gap-2">
          <FieldLabel className="h-5" htmlFor={cameraFieldId}>
            <Camera aria-hidden="true" className="size-4 text-muted-foreground" />
            {messages.camera}
          </FieldLabel>
          <Select
            disabled={cameras.length === 0}
            onValueChange={setCameraId}
            value={cameraId || undefined}
          >
            <SelectTrigger id={cameraFieldId} aria-label={messages.chooseCamera}>
              <SelectValue placeholder={messages.noCameras} />
            </SelectTrigger>
            <SelectContent dir={locale === "fa" ? "rtl" : "ltr"}>
              {cameras.map((device, index) => (
                <SelectItem key={device.deviceId} value={device.deviceId}>
                  {device.label || `${messages.camera} ${index + 1}`}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
      </div>
      <video
        autoPlay
        className="aspect-video w-full rounded-lg bg-muted object-cover"
        muted
        playsInline
        ref={videoRef}
        aria-label={messages.camera}
      />
      <p aria-live="polite" className="text-sm text-muted-foreground">
        {previewMessage}
      </p>
    </section>
  );
}

<html><head></head><body><h1>Mahfel Design System</h1><p>Mahfel is a real-time communication and collaboration platform for meetings, conversations, classes, workshops, presentations, and informal gatherings.</p><p>The interface should feel calm, modern, human, and focused.</p><p>Mahfel is not an "education UI", nor should it visually resemble enterprise dashboard software. Educational use is one use case among many.</p><p>The visual system should work equally well for:</p><ul><li><p>casual team meetings</p></li><li><p>organization meetings</p></li><li><p>classes and lectures</p></li><li><p>workshops</p></li><li><p>interviews</p></li><li><p>presentations</p></li><li><p>community gatherings</p></li><li><p>private conversations</p></li></ul><hr><h1>1. Design Principles</h1><h2>Calm by default</h2><p>The interface should stay visually quiet unless something requires attention.</p><p>Avoid:</p><ul><li><p>excessive borders</p></li><li><p>excessive shadows</p></li><li><p>large areas of brand purple</p></li><li><p>unnecessary gradients</p></li><li><p>decorative animation during meetings</p></li><li><p>multiple competing accent colors</p></li></ul><p>Meeting content should remain the visual focus.</p><hr><h2>Color communicates meaning</h2><p>Do not color controls merely for decoration.</p><p>Use color to communicate:</p><ul><li><p>active state</p></li><li><p>selected state</p></li><li><p>destructive action</p></li><li><p>connection state</p></li><li><p>warning</p></li><li><p>error</p></li><li><p>user attention</p></li></ul><p>Most interface elements should remain neutral.</p><hr><h2>Content first</h2><p>During a session, the hierarchy is:</p><ol><li><p>people/content being presented</p></li><li><p>active conversation</p></li><li><p>critical controls</p></li><li><p>contextual information</p></li><li><p>navigation</p></li></ol><p>Chrome should disappear into the background.</p><hr><h2>Progressive disclosure</h2><p>Do not expose every action at once.</p><p>Common actions stay visible.</p><p>Less common actions belong in contextual menus, popovers, command menus, or secondary panels.</p><p>Example:</p><p>Visible meeting controls:</p><ul><li><p>microphone</p></li><li><p>camera</p></li><li><p>screen share</p></li><li><p>reactions</p></li><li><p>chat</p></li><li><p>participants</p></li><li><p>leave</p></li></ul><p>Secondary actions:</p><ul><li><p>device settings</p></li><li><p>captions</p></li><li><p>recording</p></li><li><p>layout</p></li><li><p>statistics</p></li><li><p>moderation</p></li><li><p>permissions</p></li><li><p>diagnostics</p></li></ul><hr><h2>Familiar, but distinctly Mahfel</h2><p>Users already understand products such as Meet, Zoom, Teams, Discord, Slack, and Telegram.</p><p>Do not reinvent established interaction patterns without a strong reason.</p><p>Mahfel's identity should come from:</p><ul><li><p>typography</p></li><li><p>spacing</p></li><li><p>motion</p></li><li><p>Persian-aware layout</p></li><li><p>iconography</p></li><li><p>color</p></li><li><p>room/session model</p></li><li><p>subtle brand details</p></li></ul><p>—not unusual UX for its own sake.</p><hr><h1>2. Product Vocabulary</h1><p>Use neutral collaboration vocabulary.</p><p>Preferred concepts:</p><ul><li><p>Organization</p></li><li><p>Space</p></li><li><p>Room</p></li><li><p>Session</p></li><li><p>Participant</p></li><li><p>Host</p></li><li><p>Moderator</p></li><li><p>Member</p></li><li><p>Guest</p></li><li><p>Conversation</p></li><li><p>Chat</p></li><li><p>Screen share</p></li><li><p>Recording</p></li><li><p>Notes</p></li><li><p>Files</p></li></ul><p>Avoid globally encoding educational terminology such as:</p><ul><li><p>Classroom</p></li><li><p>Teacher</p></li><li><p>Student</p></li><li><p>Lesson</p></li></ul><p>These concepts may exist inside education-specific configurations.</p><hr><h1>3. Brand Personality</h1><p>Mahfel should feel:</p><ul><li><p>warm</p></li><li><p>composed</p></li><li><p>modern</p></li><li><p>trustworthy</p></li><li><p>conversational</p></li><li><p>technically competent</p></li><li><p>culturally aware</p></li></ul><p>Mahfel should not feel:</p><ul><li><p>corporate-heavy</p></li><li><p>childish</p></li><li><p>futuristic-for-the-sake-of-it</p></li><li><p>overly playful</p></li><li><p>academic</p></li><li><p>sterile</p></li><li><p>aggressively "AI"</p></li></ul><hr><h1>4. Color System</h1><p>Mahfel uses a neutral UI with an indigo-violet brand accent.</p><p>Brand purple should generally occupy a small portion of any screen.</p><p>Target visual balance:</p><ul><li><p>~80% neutral surfaces</p></li><li><p>~15% brand/accent</p></li><li><p>~5% semantic states</p></li></ul><h2>Core Palette</h2><pre><code class="language-css">:root {
  --mahfel-50: #f5f3ff;
  --mahfel-100: #ede9fe;
  --mahfel-200: #ddd6fe;
  --mahfel-300: #c4b5fd;
  --mahfel-400: #a78bfa;
  --mahfel-500: #8068f2;
  --mahfel-600: #6952e8;
  --mahfel-700: #5842d3;
  --mahfel-800: #43349d;
  --mahfel-900: #302673;
  --mahfel-950: #1d174b;
}
</code></pre><p>Primary brand color:</p><pre><code class="language-css">#6952E8
</code></pre><hr><h1>5. Semantic Colors</h1><p>Never reference raw colors directly inside components when a semantic token exists.</p><h2>Light</h2><pre><code class="language-css">:root {
  --background: #fafafc;
  --foreground: #181725;

--card: #ffffff;
--card-foreground: #181725;

--popover: #ffffff;
--popover-foreground: #181725;

--primary: #6952e8;
--primary-foreground: #ffffff;

--secondary: #f4f3f8;
--secondary-foreground: #252431;

--muted: #f4f3f8;
--muted-foreground: #706e7c;

--accent: #efecff;
--accent-foreground: #43349d;

--border: #e4e2ea;
--input: #e4e2ea;
--ring: #8068f2;

--destructive: #e5484d;
--destructive-foreground: #ffffff;

--success: #2f9e6f;
--success-foreground: #ffffff;

--warning: #d99022;
--warning-foreground: #181725;
}
</code></pre><h2>Dark</h2><pre><code class="language-css">.dark {
--background: #101018;
--foreground: #f4f3f8;

--card: #181824;
--card-foreground: #f4f3f8;

--popover: #1b1b27;
--popover-foreground: #f4f3f8;

--primary: #8068f2;
--primary-foreground: #ffffff;

--secondary: #20202d;
--secondary-foreground: #ecebf2;

--muted: #20202d;
--muted-foreground: #a5a3b0;

--accent: #292345;
--accent-foreground: #dcd5fc;

--border: #30303f;
--input: #30303f;
--ring: #a997f5;

--destructive: #ff6369;
--destructive-foreground: #ffffff;

--success: #46c98c;
--success-foreground: #101018;

--warning: #f2b84b;
--warning-foreground: #101018;
}
</code></pre><hr><h1>6. Tailwind</h1><p>Prefer semantic tokens.</p><p>Good:</p><pre><code class="language-tsx">&lt;Button className="bg-primary text-primary-foreground"&gt;
Join room
&lt;/Button&gt;
</code></pre><p>Bad:</p><pre><code class="language-tsx">&lt;Button className="bg-[#6952E8] text-white"&gt;
Join room
&lt;/Button&gt;
</code></pre><p>Good:</p><pre><code class="language-tsx">&lt;div className="border-border bg-card text-card-foreground" /&gt;
</code></pre><p>Bad:</p><pre><code class="language-tsx">&lt;div className="border-gray-200 bg-white text-gray-900" /&gt;
</code></pre><p>Raw palette values are acceptable only inside:</p><ul><li><p>illustrations</p></li><li><p>brand assets</p></li><li><p>charts</p></li><li><p>visualization systems</p></li><li><p>exceptional branded experiences</p></li></ul><hr><h1>7. shadcn/ui</h1><p>Use shadcn as infrastructure, not as Mahfel's visual identity.</p><p>Do not accept component defaults blindly.</p><p>Every component should respect:</p><ul><li><p>Mahfel radius</p></li><li><p>spacing</p></li><li><p>colors</p></li><li><p>RTL behavior</p></li><li><p>focus behavior</p></li><li><p>motion</p></li><li><p>density</p></li></ul><p>Prefer composition over introducing custom primitives when a shadcn/Radix primitive already solves accessibility correctly.</p><p>Typical primitives:</p><ul><li><p>Button</p></li><li><p>Dialog</p></li><li><p>DropdownMenu</p></li><li><p>Popover</p></li><li><p>Tooltip</p></li><li><p>Sheet</p></li><li><p>Tabs</p></li><li><p>Command</p></li><li><p>ScrollArea</p></li><li><p>Avatar</p></li><li><p>ContextMenu</p></li><li><p>Toggle</p></li><li><p>ToggleGroup</p></li><li><p>Select</p></li></ul><hr><h1>8. Border Radius</h1><p>Mahfel should feel soft but not bubbly.</p><p>Recommended scale:</p><pre><code class="language-css">--radius-xs: 4px;
--radius-sm: 6px;
--radius-md: 8px;
--radius-lg: 12px;
--radius-xl: 16px;
--radius-full: 9999px;
</code></pre><p>Typical use:</p><ul><li><p>buttons: <code inline="">8px</code></p></li><li><p>inputs: <code inline="">8px</code></p></li><li><p>cards: <code inline="">12px</code></p></li><li><p>dialogs: <code inline="">16px</code></p></li><li><p>video tiles: <code inline="">12px</code></p></li><li><p>pills: full radius</p></li></ul><p>Avoid excessive pill-shaped UI.</p><hr><h1>9. Spacing</h1><p>Use Tailwind's spacing scale.</p><p>Primary rhythm:</p><pre><code class="language-text">4
8
12
16
20
24
32
40
48
64
</code></pre><p>Use tighter spacing for controls and denser spacing inside meetings.</p><p>Use more whitespace in:</p><ul><li><p>dashboards</p></li><li><p>room discovery</p></li><li><p>settings</p></li><li><p>onboarding</p></li></ul><hr><h1>10. Typography</h1><p>Mahfel must support Persian and English equally well.</p><p>Do not treat Persian as translated English.</p><p>Typography must be tested independently in RTL.</p><p>Recommended structure:</p><pre><code class="language-css">font-family:
"Vazirmatn",
"Inter",
system-ui,
sans-serif;
</code></pre><p>If Vazirmatn is unavailable, choose another high-quality Persian UI font.</p><p>Avoid mixing visibly incompatible Persian and Latin fonts.</p><h2>Type Scale</h2><pre><code class="language-text">12px — metadata
13px — compact labels
14px — body small
16px — normal body
18px — emphasized body
20px — section title
24px — page title
30px — major heading
36px+ — marketing only
</code></pre><p>Use weight rather than size excessively.</p><p>Typical weights:</p><ul><li><p>400 normal</p></li><li><p>500 UI emphasis</p></li><li><p>600 titles</p></li><li><p>700 rarely</p></li></ul><p>Avoid <code inline="">font-bold</code> everywhere.</p><hr><h1>11. Persian and RTL</h1><p>RTL support is a first-class requirement.</p><p>Do not implement RTL as a final translation pass.</p><p>Use logical CSS properties wherever possible.</p><p>Prefer:</p><pre><code class="language-css">margin-inline-start
margin-inline-end
padding-inline
inset-inline-start
border-inline-start
</code></pre><p>Avoid:</p><pre><code class="language-css">margin-left
margin-right
padding-left
padding-right
</code></pre><p>unless physical direction is truly intentional.</p><p>Tailwind should prefer directional-safe patterns.</p><hr><h1>12. Directionality Rules</h1><p>Some elements mirror in RTL.</p><p>Examples:</p><ul><li><p>back arrow</p></li><li><p>next/previous</p></li><li><p>breadcrumbs</p></li><li><p>sidebar placement</p></li><li><p>directional navigation</p></li><li><p>chevrons</p></li></ul><p>Some elements do NOT mirror.</p><p>Examples:</p><ul><li><p>play icon</p></li><li><p>microphone</p></li><li><p>camera</p></li><li><p>volume</p></li><li><p>download</p></li><li><p>upload</p></li><li><p>checkmark</p></li><li><p>power</p></li><li><p>media controls with established meaning</p></li></ul><p>Do not mechanically mirror every SVG.</p><hr><h1>13. Numbers</h1><p>Persian interfaces may contain:</p><ul><li><p>Latin numbers</p></li><li><p>Persian numbers</p></li><li><p>phone numbers</p></li><li><p>timestamps</p></li><li><p>IDs</p></li><li><p>URLs</p></li><li><p>code</p></li><li><p>technical metrics</p></li></ul><p>Do not force Persian digits into technical identifiers.</p><p>Examples:</p><pre><code class="language-text">Room ID: 843 126
1080p
24 fps
42 ms
Opus
VP9
</code></pre><p>should remain technically readable.</p><hr><h1>14. Icon System</h1><p>Mahfel uses its own minimal icon set.</p><p>Do not visually copy Lucide.</p><p>The icon language should feel slightly influenced by Persian geometry without becoming decorative.</p><h2>Base Geometry</h2><pre><code class="language-text">Canvas: 24 × 24
Stroke: 1.75px
Linecap: round
Linejoin: round
Optical area: ~20 × 20
</code></pre><p>Primary sizes:</p><pre><code class="language-text">16px — metadata
20px — standard UI
24px — prominent actions
28–32px — meeting controls
</code></pre><p>SVG components must use:</p><pre><code class="language-svg">stroke="currentColor"
</code></pre><p>or:</p><pre><code class="language-svg">fill="currentColor"
</code></pre><p>Do not hardcode Mahfel purple in the SVG.</p><hr><h1>15. Core Icon Set</h1><p>Minimum first-party set:</p><h2>Navigation</h2><ul><li><p>Home</p></li><li><p>Rooms</p></li><li><p>Sessions</p></li><li><p>Chat</p></li><li><p>People</p></li><li><p>Search</p></li><li><p>Notifications</p></li><li><p>Settings</p></li><li><p>Profile</p></li><li><p>More</p></li><li><p>Back</p></li><li><p>Close</p></li></ul><h2>Rooms / Sessions</h2><ul><li><p>Room</p></li><li><p>Create room</p></li><li><p>Join</p></li><li><p>Invite</p></li><li><p>Invite link</p></li><li><p>Participants</p></li><li><p>Host</p></li><li><p>Moderator</p></li><li><p>Calendar</p></li><li><p>Schedule</p></li><li><p>Waiting room</p></li><li><p>Live</p></li><li><p>Record</p></li><li><p>Archive</p></li><li><p>Leave</p></li></ul><h2>Audio / Video</h2><ul><li><p>Microphone</p></li><li><p>Microphone off</p></li><li><p>Camera</p></li><li><p>Camera off</p></li><li><p>Screen share</p></li><li><p>Stop screen share</p></li><li><p>Speaker</p></li><li><p>Speaker muted</p></li><li><p>Headphones</p></li><li><p>Device settings</p></li><li><p>Picture in picture</p></li><li><p>Fullscreen</p></li><li><p>Quality</p></li><li><p>Captions</p></li></ul><h2>Chat</h2><ul><li><p>Message</p></li><li><p>Send</p></li><li><p>Reply</p></li><li><p>Thread</p></li><li><p>Reaction</p></li><li><p>Emoji</p></li><li><p>Mention</p></li><li><p>Pin</p></li><li><p>Attach</p></li><li><p>File</p></li><li><p>Image</p></li><li><p>Raise hand</p></li></ul><h2>Utility</h2><ul><li><p>Copy</p></li><li><p>Edit</p></li><li><p>Delete</p></li><li><p>Check</p></li><li><p>Warning</p></li><li><p>Info</p></li><li><p>Help</p></li><li><p>Download</p></li><li><p>Upload</p></li><li><p>Link</p></li><li><p>Lock</p></li><li><p>Unlock</p></li><li><p>Shield</p></li></ul><hr><h1>16. Buttons</h1><h2>Primary</h2><p>Use for the dominant action.</p><p>Examples:</p><ul><li><p>Create room</p></li><li><p>Join</p></li><li><p>Save</p></li><li><p>Start session</p></li></ul><pre><code class="language-text">background: primary
foreground: white
</code></pre><p>Do not place multiple primary buttons beside each other unless absolutely necessary.</p><hr><h2>Secondary</h2><p>For supporting actions.</p><pre><code class="language-text">background: secondary
foreground: foreground
</code></pre><hr><h2>Ghost</h2><p>Preferred for toolbar actions.</p><pre><code class="language-text">background: transparent
hover: muted
</code></pre><p>Meeting toolbars should use ghost-style controls heavily.</p><hr><h2>Destructive</h2><p>Reserved for destructive actions.</p><p>Examples:</p><ul><li><p>end session</p></li><li><p>delete room</p></li><li><p>remove participant</p></li><li><p>leave organization</p></li></ul><p>Do not use red for generic disabled states.</p><hr><h1>17. Meeting Controls</h1><p>Meeting controls are a special interaction environment.</p><p>They must remain:</p><ul><li><p>obvious</p></li><li><p>reachable</p></li><li><p>high contrast</p></li><li><p>consistent</p></li><li><p>difficult to trigger accidentally</p></li></ul><p>Recommended order conceptually:</p><pre><code class="language-text">Mic
Camera
Screen Share
Reactions
More

Chat
Participants

Leave / End
</code></pre><p>The exact responsive layout may change by viewport.</p><hr><h1>18. Meeting Control States</h1><h2>Microphone on</h2><p>Neutral.</p><h2>Microphone off</h2><p>Use destructive/red semantic state.</p><p>Muting yourself is not an "error", but red is already strongly understood as disabled audio in conferencing software.</p><h2>Camera on</h2><p>Neutral.</p><h2>Camera off</h2><p>Red accent or disabled-state treatment.</p><h2>Screen sharing</h2><p>Use primary purple while active.</p><h2>Recording</h2><p>Use a clear red recording indicator.</p><p>Recording status must not depend only on color.</p><p>Display:</p><pre><code class="language-text">● Recording
</code></pre><p>with textual or icon support.</p><hr><h1>19. Leave vs End Session</h1><p>These are not the same action.</p><p>Participant:</p><pre><code class="language-text">Leave
</code></pre><p>Host:</p><pre><code class="language-text">Leave
End for everyone
</code></pre><p>"End for everyone" requires confirmation.</p><p>Never place destructive confirmation as the default focused action.</p><hr><h1>20. Video Tiles</h1><p>Video tiles should prioritize people.</p><p>Avoid heavy framing.</p><p>Recommended:</p><pre><code class="language-text">border-radius: 12px
background: dark neutral
</code></pre><p>Overlay elements:</p><ul><li><p>name</p></li><li><p>mute status</p></li><li><p>speaking state</p></li><li><p>connection warning</p></li><li><p>moderation state</p></li></ul><p>Keep overlays minimal.</p><hr><h1>21. Active Speaker</h1><p>Do not use large glowing neon borders.</p><p>Preferred:</p><ul><li><p>subtle primary border</p></li><li><p>soft outer highlight</p></li><li><p>optional speaking indicator</p></li></ul><p>Example:</p><pre><code class="language-text">1–2px primary border
</code></pre><p>Avoid distracting animation while someone is speaking.</p><hr><h1>22. Avatar Fallback</h1><p>When video is disabled:</p><pre><code class="language-text">avatar
name
audio state
</code></pre><p>Use deterministic avatar colors.</p><p>Avoid random colors changing between sessions.</p><hr><h1>23. Participant Grid</h1><p>Layouts should optimize for:</p><ul><li><p>participant count</p></li><li><p>available space</p></li><li><p>presentation mode</p></li><li><p>active speaker</p></li><li><p>screen share</p></li><li><p>pinned participants</p></li></ul><p>Layouts:</p><ul><li><p>Grid</p></li><li><p>Speaker</p></li><li><p>Presentation</p></li><li><p>Sidebar</p></li><li><p>Picture-in-picture</p></li></ul><p>Do not expose unnecessary layout complexity to casual users.</p><hr><h1>24. Screen Sharing</h1><p>Screen sharing is content-first.</p><p>Shared content should receive the largest available area.</p><p>Participant videos become secondary.</p><p>The presenter should always have an obvious indication that sharing is active.</p><hr><h1>25. Chat</h1><p>Meeting chat should feel lightweight.</p><p>Use:</p><ul><li><p>compact messages</p></li><li><p>subtle grouping</p></li><li><p>timestamps when useful</p></li><li><p>reactions without excessive decoration</p></li></ul><p>Avoid oversized chat bubbles.</p><p>For desktop meeting mode, chat should usually appear in a side panel rather than covering video.</p><hr><h1>26. Side Panels</h1><p>Chat, participants, details, and similar tools should use the same panel system.</p><p>Desktop:</p><pre><code class="language-text">320–400px
</code></pre><p>Mobile:</p><pre><code class="language-text">full-screen sheet
</code></pre><p>Only one major meeting side panel should normally be open at once.</p><hr><h1>27. Notifications</h1><p>Use three levels:</p><h2>Toast</h2><p>Temporary confirmation.</p><p>Example:</p><pre><code class="language-text">Invite link copied
</code></pre><h2>Inline status</h2><p>State attached to a component.</p><p>Example:</p><pre><code class="language-text">Connecting…
</code></pre><h2>Dialog</h2><p>Requires action or contains significant consequences.</p><p>Example:</p><pre><code class="language-text">End session for everyone?
</code></pre><p>Do not use dialogs for trivial information.</p><hr><h1>28. Connection States</h1><p>Mahfel is a real-time system, so connection state is part of the design system.</p><p>Possible states:</p><pre><code class="language-text">connecting
connected
reconnecting
degraded
offline
failed
</code></pre><p>Do not expose raw WebRTC terminology to normal users.</p><p>Bad:</p><pre><code class="language-text">ICE connection failed
</code></pre><p>Better:</p><pre><code class="language-text">Connection lost. Reconnecting…
</code></pre><p>Advanced diagnostics may expose technical details separately.</p><hr><h1>29. Latency / Quality</h1><p>Do not constantly display technical metrics.</p><p>Normal users should see:</p><ul><li><p>Good</p></li><li><p>Unstable</p></li><li><p>Poor</p></li></ul><p>Advanced diagnostics may expose:</p><pre><code class="language-text">RTT
jitter
packet loss
bitrate
codec
resolution
FPS
</code></pre><hr><h1>30. Empty States</h1><p>Empty states should be concise.</p><p>Example:</p><pre><code class="language-text">No rooms yet.

Create a room to start a conversation.
</code></pre><p>Avoid large decorative illustrations for every empty state.</p><hr><h1>31. Loading</h1><p>Prefer skeletons when layout is known.</p><p>Use spinner only when:</p><ul><li><p>action duration is unknown</p></li><li><p>skeleton is inappropriate</p></li><li><p>control-level progress is needed</p></li></ul><p>Never block the whole UI unnecessarily.</p><hr><h1>32. Motion</h1><p>Motion should communicate state.</p><p>Recommended duration:</p><pre><code class="language-text">100–150ms — hover / press
150–200ms — menus / popovers
200–250ms — dialogs / sheets
250–350ms — larger transitions
</code></pre><p>Preferred easing:</p><pre><code class="language-css">cubic-bezier(0.2, 0, 0, 1)
</code></pre><p>Avoid springy animation for core meeting controls.</p><hr><h1>33. Reduced Motion</h1><p>Always respect:</p><pre><code class="language-css">@media (prefers-reduced-motion: reduce)
</code></pre><p>Disable:</p><ul><li><p>decorative movement</p></li><li><p>unnecessary transforms</p></li><li><p>continuous ambient animation</p></li></ul><p>Preserve state changes.</p><hr><h1>34. Accessibility</h1><p>Target WCAG AA minimum.</p><p>All controls require:</p><ul><li><p>keyboard access</p></li><li><p>visible focus state</p></li><li><p>accessible label</p></li><li><p>appropriate ARIA semantics</p></li><li><p>sufficient contrast</p></li></ul><p>Icon-only buttons must include an accessible name.</p><p>Example:</p><pre><code class="language-tsx">&lt;Button
variant="ghost"
size="icon"
aria-label="Mute microphone"
&gt;
&lt;MicrophoneOff /&gt;
&lt;/Button&gt;
</code></pre><hr><h1>35. Tooltips</h1><p>All unfamiliar icon-only actions should have tooltips.</p><p>Meeting controls should generally expose:</p><pre><code class="language-text">Mute microphone
Turn camera off
Share screen
Open chat
Participants
More options
</code></pre><p>Tooltips supplement accessible labels; they do not replace them.</p><hr><h1>36. Focus States</h1><p>Use visible focus rings.</p><pre><code class="language-text">2px ring
2px offset
</code></pre><p>Use <code inline="">--ring</code>.</p><p>Do not remove browser focus behavior without replacing it.</p><hr><h1>37. Hover Is Optional</h1><p>Every interaction must work without hover.</p><p>This matters for:</p><ul><li><p>touch</p></li><li><p>mobile</p></li><li><p>tablets</p></li><li><p>accessibility</p></li><li><p>hybrid devices</p></li></ul><p>Do not hide essential controls exclusively behind hover.</p><hr><h1>38. Responsive Layout</h1><p>Primary targets:</p><pre><code class="language-text">mobile
tablet
desktop
large desktop
</code></pre><p>Meeting UX should not simply shrink desktop layout.</p><p>Mobile priorities:</p><ol><li><p>current speaker/content</p></li><li><p>microphone</p></li><li><p>camera</p></li><li><p>leave</p></li><li><p>essential contextual controls</p></li></ol><p>Secondary actions move into sheets/menus.</p><hr><h1>39. Desktop Navigation</h1><p>For the main application, prefer a stable sidebar/navigation rail.</p><p>Typical structure:</p><pre><code class="language-text">Mahfel

Home
Rooms
Sessions
Messages

────────

Organization

────────

Settings
Profile
</code></pre><p>Avoid deeply nested navigation.</p><hr><h1>40. Command Palette</h1><p>Mahfel may provide a command palette for power users.</p><p>Examples:</p><pre><code class="language-text">Create room
Join room
Search rooms
Open settings
Mute microphone
Toggle camera
Open chat
</code></pre><p>Keyboard-first workflows are encouraged on desktop.</p><hr><h1>41. Keyboard Shortcuts</h1><p>Meeting actions should support keyboard shortcuts where practical.</p><p>Examples:</p><pre><code class="language-text">M — microphone
V — camera
C — chat
P — participants
</code></pre><p>Avoid overriding common browser/system shortcuts.</p><p>Shortcuts must be discoverable.</p><hr><h1>42. Cards</h1><p>Cards are for meaningful grouping, not every piece of content.</p><p>Avoid "card soup".</p><p>Good candidates:</p><ul><li><p>room preview</p></li><li><p>upcoming session</p></li><li><p>organization</p></li><li><p>settings group</p></li></ul><p>Bad candidates:</p><ul><li><p>every row</p></li><li><p>every field</p></li><li><p>every navigation item</p></li></ul><hr><h1>43. Shadows</h1><p>Use very subtle shadows.</p><p>Default surfaces should usually rely on:</p><ul><li><p>contrast</p></li><li><p>border</p></li><li><p>spacing</p></li></ul><p>rather than shadows.</p><p>Dialogs/popovers may use elevation.</p><p>Avoid large blurred SaaS-style shadows.</p><hr><h1>44. Gradients</h1><p>Brand gradients are reserved for:</p><ul><li><p>marketing</p></li><li><p>onboarding</p></li><li><p>logo treatment</p></li><li><p>rare premium brand moments</p></li></ul><p>Do not use gradients extensively inside the working application.</p><p>Meeting controls should remain flat.</p><hr><h1>45. Brand Presence</h1><p>Mahfel should be recognizable without placing the logo everywhere.</p><p>Brand identity comes from:</p><ul><li><p>color</p></li><li><p>icons</p></li><li><p>typography</p></li><li><p>radius</p></li><li><p>spacing</p></li><li><p>interaction</p></li><li><p>motion</p></li><li><p>RTL quality</p></li></ul><p>The interface itself should carry the brand.</p><hr><h1>46. Persianized Design</h1><p>"Persianized" does not mean adding traditional ornamentation to every component.</p><p>Use Persian influence subtly through:</p><ul><li><p>custom icon geometry</p></li><li><p>typography</p></li><li><p>language-aware spacing</p></li><li><p>motion</p></li><li><p>logo curves</p></li><li><p>illustration style</p></li><li><p>culturally appropriate visual balance</p></li></ul><p>Avoid:</p><ul><li><p>ornamental borders in functional UI</p></li><li><p>fake calligraphy</p></li><li><p>excessive traditional motifs</p></li><li><p>stereotypical Persian decoration</p></li></ul><p>Decorative Persian visual language belongs primarily in:</p><ul><li><p>campaigns</p></li><li><p>illustrations</p></li><li><p>onboarding</p></li><li><p>landing pages</p></li><li><p>special events</p></li></ul><p>not core controls.</p><hr><h1>47. Room Cards</h1><p>A room card may include:</p><pre><code class="language-text">room name
organization/space
participant presence
next scheduled session
privacy state
primary action
</code></pre><p>Avoid displaying every available metadata field.</p><hr><h1>48. Presence</h1><p>Presence should be subtle.</p><p>Possible states:</p><pre><code class="language-text">online
away
busy
offline
in session
</code></pre><p>Use dot indicators sparingly.</p><p>Never rely solely on color; expose text in detailed contexts.</p><hr><h1>49. Status Language</h1><p>Prefer human language.</p><p>Bad:</p><pre><code class="language-text">RTC transport disconnected
</code></pre><p>Better:</p><pre><code class="language-text">Connection interrupted
</code></pre><p>Bad:</p><pre><code class="language-text">Producer paused
</code></pre><p>Better:</p><pre><code class="language-text">Camera paused
</code></pre><p>Technical terminology belongs in diagnostics.</p><hr><h1>50. Error Messages</h1><p>Errors should contain:</p><ol><li><p>what happened</p></li><li><p>whether the user needs to act</p></li><li><p>recovery action when possible</p></li></ol><p>Example:</p><pre><code class="language-text">We couldn't access your microphone.

Check your browser permissions and try again.
</code></pre><p>Avoid:</p><pre><code class="language-text">PermissionDeniedError
</code></pre><hr><h1>51. Dangerous Actions</h1><p>Require confirmation for actions with broad impact:</p><ul><li><p>delete room</p></li><li><p>end session for everyone</p></li><li><p>remove organization</p></li><li><p>revoke recording</p></li><li><p>permanently delete account</p></li></ul><p>Do not confirm routine reversible actions.</p><hr><h1>52. Forms</h1><p>Labels remain visible.</p><p>Do not use placeholders as labels.</p><p>Good:</p><pre><code class="language-text">Room name
[ Weekly engineering sync ]
</code></pre><p>Bad:</p><pre><code class="language-text">[ Enter room name... ]
</code></pre><p>Use descriptions only where ambiguity exists.</p><hr><h1>53. Settings</h1><p>Settings should be grouped by domain:</p><pre><code class="language-text">Profile
Audio &amp; Video
Notifications
Appearance
Accessibility
Privacy
Organization
Advanced
</code></pre><p>Advanced technical settings should not clutter normal preferences.</p><hr><h1>54. Appearance</h1><p>Support user-selectable Light, Dark, and System appearance modes. System is the default and follows the operating system preference until the user chooses a mode.</p><p>Persist an explicit appearance choice across visits. Apply the saved mode before the page is first painted to prevent a theme flash.</p><p>Use the semantic light and dark token sets in Section 5 for every application surface. Components must not depend on a particular theme or use raw colors when a semantic token exists.</p><p>Keep the appearance control accessible by keyboard, visibly focused, localized, and usable in RTL layouts.</p><p>Do not assume dark mode is preferred for meetings. Meeting backgrounds may remain dark while the rest of the application follows the user's selected appearance.</p><p>Both themes must be treated as first-class and checked for contrast, focus visibility, and readable controls.</p><hr><h1>55. Meeting Background</h1><p>Default meeting background should be a neutral dark tone.</p><p>Example:</p><pre><code class="language-css">#101018
</code></pre><p>Dark backgrounds reduce visual distraction around video tiles.</p><p>The rest of the application may follow the user's selected theme.</p><hr><h1>56. Recording and Privacy</h1><p>Recording state must be unmistakable.</p><p>Before recording begins, participants should receive clear notice according to product policy.</p><p>Never make recording visually subtle.</p><hr><h1>57. AI Features</h1><p>AI should not dictate Mahfel's core visual identity.</p><p>AI functionality may use:</p><ul><li><p>subtle Mahfel violet</p></li><li><p>Parto mascot</p></li><li><p>soft glow</p></li><li><p>dedicated surface treatment</p></li></ul><p>but the base product must remain coherent without AI.</p><p>Do not place sparkle icons everywhere.</p><hr><h1>58. Parto</h1><p>Parto is Mahfel's AI companion.</p><p>Parto is not a generic navigation icon and should not appear in every surface.</p><p>Use Parto for:</p><ul><li><p>AI chat</p></li><li><p>summarization</p></li><li><p>search assistance</p></li><li><p>meeting intelligence</p></li><li><p>onboarding/help</p></li><li><p>AI-generated actions</p></li></ul><p>Parto should not replace conventional UI symbols.</p><hr><h1>59. Copy Style</h1><p>UI copy should be:</p><ul><li><p>short</p></li><li><p>direct</p></li><li><p>conversational</p></li><li><p>neutral</p></li><li><p>useful</p></li></ul><p>Prefer:</p><pre><code class="language-text">Join room
</code></pre><p>over:</p><pre><code class="language-text">Proceed to room
</code></pre><p>Prefer:</p><pre><code class="language-text">Microphone is off
</code></pre><p>over:</p><pre><code class="language-text">Your microphone has been deactivated
</code></pre><hr><h1>60. English and Persian Copy</h1><p>Do not translate mechanically.</p><p>Persian wording should sound natural in Persian.</p><p>English wording should sound natural in English.</p><p>Layout should accommodate different text lengths without breaking.</p><hr><h1>61. Dates and Times</h1><p>Display according to user locale.</p><p>Support:</p><ul><li><p>Gregorian</p></li><li><p>potentially Jalali where product requirements call for it</p></li></ul><p>Do not embed calendar assumptions inside reusable components.</p><p>Use Intl APIs whenever possible.</p><hr><h1>62. Component Ownership</h1><p>Reusable UI belongs in:</p><pre><code class="language-text">components/ui/
</code></pre><p>Mahfel domain components belong in:</p><pre><code class="language-text">components/mahfel/
</code></pre><p>Example:</p><pre><code class="language-text">components/
├── ui/
│ ├── button.tsx
│ ├── dialog.tsx
│ └── tooltip.tsx
│
└── mahfel/
├── meeting-controls.tsx
├── participant-tile.tsx
├── room-card.tsx
└── connection-indicator.tsx
</code></pre><p>Do not modify low-level primitives for one-off feature behavior.</p><hr><h1>63. Class Composition</h1><p>Use <code inline="">cn()</code> for conditional Tailwind composition.</p><pre><code class="language-tsx">cn(
"rounded-lg border bg-card",
active &amp;&amp; "border-primary",
disabled &amp;&amp; "opacity-50"
)
</code></pre><p>Avoid manually concatenating large class strings.</p><hr><h1>64. Variants</h1><p>Use <code inline="">class-variance-authority</code> for reusable components with meaningful variants.</p><p>Example:</p><pre><code class="language-text">variant:
default
secondary
ghost
destructive

size:
sm
default
lg
icon
</code></pre><p>Do not create variants for arbitrary stylistic differences.</p><hr><h1>65. Icon Buttons</h1><p>Minimum interactive target:</p><pre><code class="language-text">40 × 40px
</code></pre><p>Meeting controls:</p><pre><code class="language-text">44–48px minimum
</code></pre><p>The visible icon may remain 20–24px.</p><p>Touch target matters more than icon size.</p><hr><h1>66. Z-Index</h1><p>Maintain a deliberate layer scale.</p><p>Example:</p><pre><code class="language-text">0 page
10 sticky elements
20 overlays
30 dropdown
40 popover
50 modal
60 toast
70 critical system overlay
</code></pre><p>Avoid arbitrary values such as:</p><pre><code class="language-css">z-[999999]
</code></pre><hr><h1>67. Design Review Checklist</h1><p>Before shipping a UI change, verify:</p><h3>Visual</h3><ul><li><p>Is the hierarchy obvious?</p></li><li><p>Is purple being overused?</p></li><li><p>Are spacing and radius consistent?</p></li><li><p>Does the screen work in dark mode?</p></li></ul><h3>Interaction</h3><ul><li><p>Is the primary action obvious?</p></li><li><p>Are destructive actions clearly separated?</p></li><li><p>Are uncommon actions progressively disclosed?</p></li></ul><h3>Accessibility</h3><ul><li><p>Keyboard accessible?</p></li><li><p>Focus visible?</p></li><li><p>Icon-only actions labeled?</p></li><li><p>Contrast acceptable?</p></li><li><p>Reduced motion supported?</p></li></ul><h3>RTL</h3><ul><li><p>Tested in Persian?</p></li><li><p>Correct reading direction?</p></li><li><p>Correct icon mirroring?</p></li><li><p>No hard-coded left/right assumptions?</p></li></ul><h3>Responsive</h3><ul><li><p>Works on narrow screens?</p></li><li><p>Touch targets large enough?</p></li><li><p>Panels and dialogs adapt properly?</p></li></ul><h3>Real-time</h3><ul><li><p>Loading state?</p></li><li><p>reconnecting state?</p></li><li><p>offline state?</p></li><li><p>permission denied state?</p></li><li><p>device failure state?</p></li></ul><hr><h1>68. Final Rule</h1><p>When choosing between visual novelty and clarity:</p><p><strong>choose clarity.</strong></p><p>When choosing between more UI and less UI:</p><p><strong>prefer less UI.</strong></p><p>When choosing whether an interaction should be "Persianized":</p><p><strong>preserve familiar behavior and Persianize the visual language, not the usability model.</strong></p><p>Mahfel should feel like a product built naturally for Persian-speaking users while remaining completely understandable and comfortable internationally.</p></body></html>

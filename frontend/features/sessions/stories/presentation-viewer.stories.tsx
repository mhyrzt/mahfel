import { useState, type ComponentProps } from "react";
import type { Meta, StoryObj } from "@storybook/nextjs";

import { PresentationViewer } from "../components/presentation-viewer";

function Demo(args: ComponentProps<typeof PresentationViewer>) {
  const [page, setPage] = useState(args.currentPage);
  const [count, setCount] = useState(args.totalPages);
  return (
    <PresentationViewer
      {...args}
      currentPage={page}
      totalPages={count}
      onPageChange={setPage}
      onStop={() => setCount(0)}
    />
  );
}

const meta = {
  title: "Sessions/Presentation",
  component: PresentationViewer,
  decorators: [
    (Story) => (
      <div className="h-[min(42rem,90dvh)]">
        <Story />
      </div>
    ),
  ],
  render: (args) => <Demo {...args} />,
  args: {
    currentPage: 1,
    totalPages: 4,
    canPresent: true,
    live: true,
    locale: "en",
    onPageChange: () => undefined,
    onStop: () => undefined,
    renderPage: (page) => (
      <div className="flex aspect-video w-full max-w-4xl flex-col justify-center gap-5 rounded-lg bg-card p-10 text-foreground">
        <p className="text-sm font-medium text-primary">Research methods · {page}</p>
        <h2 className="text-4xl font-semibold tracking-tight">
          {
            [
              "Start with a question",
              "Gather the evidence",
              "Compare the findings",
              "Share what you learned",
            ][page - 1]
          }
        </h2>
        <p className="max-w-lg text-lg text-muted-foreground">
          Weekly workshop — collaborative presentation
        </p>
      </div>
    ),
  },
} satisfies Meta<typeof PresentationViewer>;

export default meta;
type Story = StoryObj<typeof meta>;
export const Presenter: Story = {};
export const Viewer: Story = { args: { canPresent: false, currentPage: 2 } };
export const Persian: Story = { args: { locale: "fa" } };
export const Empty: Story = { args: { totalPages: 0 } };
export const Unavailable: Story = { args: { failed: true } };

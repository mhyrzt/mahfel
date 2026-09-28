import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";

import { MonthCalendar } from "../components/month-calendar";

test("Persian months use Shamsi dates and compact Saturday-first weekdays", () => {
  const onMonthChange = vi.fn();
  render(
    <MonthCalendar
      activeDates={["2026-03-21"]}
      calendar="persian"
      locale="fa"
      monthDate="2026-03-21"
      onMonthChange={onMonthChange}
      onSelectDate={vi.fn()}
      selectedDate="2026-03-21"
      today="2026-03-21"
    />,
  );

  expect(screen.getByRole("button", { name: "2026-03-21" })).toHaveTextContent("۱");
  expect(screen.getByText("۱۴۰۵ فروردین")).toBeInTheDocument();
  expect(
    screen
      .getAllByText(/^[شیدسچپج]$/)
      .slice(0, 7)
      .map((item) => item.textContent),
  ).toEqual(["ش", "ی", "د", "س", "چ", "پ", "ج"]);

  fireEvent.click(screen.getByRole("button", { name: "ماه بعد" }));
  expect(onMonthChange).toHaveBeenCalledWith("2026-04-21");
});

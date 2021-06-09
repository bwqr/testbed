export function convertDateToServerDate(date: Date): string {
  const iso = date.toISOString();
  return iso.slice(0, iso.length - 5);
}

export function convertDateToLocal(date: string | Date): Date {
  if (!(date instanceof Date)) {
    date = new Date(date);
  }

  const timeOffset = date.getTimezoneOffset() * 1000 * 60;

  date.setTime(date.getTime() - timeOffset);

  return date;
}

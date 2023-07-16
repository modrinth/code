export function addReportMessage(thread, report) {
  if (!thread || !report) {
    return thread
  }
  if (
    !thread.members.some((user) => {
      return user.id === report.reporterUser.id
    })
  ) {
    thread.members.push(report.reporterUser)
  }
  if (!thread.messages.some((message) => message.id === 'original')) {
    thread.messages.push({
      id: 'original',
      author_id: report.reporterUser.id,
      body: {
        type: 'text',
        body: report.body,
        private: false,
        replying_to: null,
      },
      created: report.created,
    })
  }
  return thread
}

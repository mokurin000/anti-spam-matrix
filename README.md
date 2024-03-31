# anti-spam-matrix

This is a simple Matrix spam banning bot.

Its logic is currently straightforward:

If a user triggers the specified keyword in a certain number of consecutive messages, they will be banned.

If a user triggers the keyword in numerous (the spam_limit) consecutive messages in different groups, they will still be banned.

The bot will ban the sender of spam messages in all rooms where it has permissions.
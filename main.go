package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"time"

	"github.com/go-telegram/bot"
	"github.com/go-telegram/bot/models"
	"github.com/joho/godotenv"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"vellarity.em_bot/m/database"
)

// Send any text message to the bot after the bot has been started

type Command string

const (
	Start     Command = "start"
	Help      Command = "help"
	Info      Command = "info"
	OmsCard   Command = "omscard"
	DateBirth Command = "datebirth"
)

var DB *gorm.DB

func main() {
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt)
	defer cancel()

	var err error

	DB, err = gorm.Open(sqlite.Open("db.sqlite"), &gorm.Config{})
	if err != nil {
		panic(err)
	}
	DB.AutoMigrate(&database.User{})

	err = godotenv.Load(".env")
	if err != nil {
		panic(err)
	}
	token := os.Getenv("TOKEN")

	opts := []bot.Option{
		bot.WithMessageTextHandler("/", bot.MatchTypePrefix, handler),
	}

	b, err := bot.New(token, opts...)
	if err != nil {
		panic(err)
	}

	go notifyUser(ctx, b)

	b.Start(ctx)
}

func notifyUser(ctx context.Context, b *bot.Bot) {
	for i := 0; ; i++ {
		users := []database.User{}
		result := DB.Not("oms_card = NULL AND birth_date = NULL").Find(&users)
		if result.Error != nil {
			panic(result.Error)
		}

		for _, v := range users {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: v.ChatID,
				Text:   "Test",
			})
		}

		time.Sleep(time.Second * 30)
	}
}

func handler(ctx context.Context, b *bot.Bot, update *models.Update) {

	command := strings.Split(strings.Replace(update.Message.Text, "/", "", 1), " ")

	switch command[0] {
	case string(Help):
		b.SendMessage(ctx, &bot.SendMessageParams{
			ChatID: update.Message.Chat.ID,
			Text: fmt.Sprintf(
				"Список доступных команд: \n\n/%s - Вывести это сообщение. \n/%s - Инициализировать вас в моей базе данных. \n/%s - Вывести информацию о вас. \n/%s - Установить новый номер ПОЛИСа в формате 16 чисел. \n/%s - Установить новую дату рождения в формате ДД.ММ.ГГГГ.",
				Help, Start, Info, OmsCard, DateBirth,
			),
		})

	case string(Start):
		var count int64

		result := DB.Table("users").Where("id = ? AND chat_id = ?", update.Message.From.ID, update.Message.Chat.ID).Count(&count)
		if result.Error != nil {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Не удалось получить данные о вашем пользователе. Попробуйте позже или обратитесь к автору бота.",
			})
			break
		}
		if count == 0 {
			created := DB.Create(&database.User{
				ID:     update.Message.From.ID,
				ChatID: update.Message.Chat.ID,
			})
			if created.Error != nil || created.RowsAffected == 0 {
				b.SendMessage(ctx, &bot.SendMessageParams{
					ChatID: update.Message.Chat.ID,
					Text:   "Не удалось создать ваш профиль. Попробуйте команду ещё раз или обратитесь к автору бота.",
				})
			}

			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Ваш профиль успешно создан. Добавьте свой ПОЛИС и дату рождения при помощи соответствующих команд. Для справки вызовите `/help`",
			})
		} else {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Ваш пользователь найден в базе. Инициализация не требуется.",
			})
		}

	case string(DateBirth):
		var user database.User

		err := DB.First(&user, update.Message.From.ID).Error
		if err != nil {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Ваш пользователь не найден в базе. Попробуйте инициализировать запись при помощи команды `/start` или обратитесь к автору бота.",
			})
			break
		}

		if len(command) == 1 {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Дата должна быть прикреплена в формате ДД.ММ.ГГГГ без дополнительных символов.",
			})
			break
		}

		date, err := time.Parse("02.01.2006", command[1])
		if err != nil {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Дата должна быть прикреплена в формате ДД.ММ.ГГГГ без дополнительных символов.",
			})
			break
		}

		user.BirthDate = &date

		err = DB.Save(user).Error
		if err != nil {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Не удалось обновить дату рождения.",
			})
			break
		}

		b.SendMessage(ctx, &bot.SendMessageParams{
			ChatID: update.Message.Chat.ID,
			Text:   "Дата рождения успешно обновлёна.",
		})

	case string(OmsCard):
		var user database.User

		err := DB.First(&user, update.Message.From.ID).Error
		if err != nil {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Ваш пользователь не найден в базе. Попробуйте инициализировать запись при помощи команды `/start` или обратитесь к автору бота.",
			})
			break
		}

		if len(command) == 1 {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Полис должен быть прикреплён в формате 16 чисел без дополнительных символов.",
			})
			break
		}

		parsedCard, err := strconv.Atoi(command[1])
		if len(command[1]) != 16 || err != nil {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Полис должен быть прикреплён в формате 16 чисел без дополнительных символов.",
			})
			break
		}

		user.OmsCard = int64(parsedCard)

		err = DB.Save(user).Error
		if err != nil {
			b.SendMessage(ctx, &bot.SendMessageParams{
				ChatID: update.Message.Chat.ID,
				Text:   "Не удалось обновить ПОЛИС.",
			})
			break
		}

		b.SendMessage(ctx, &bot.SendMessageParams{
			ChatID: update.Message.Chat.ID,
			Text:   "Полис успешно обновлён.",
		})
	}
}

package todo

type Todo struct {
	ID     uint64 `json:"id" gorm:"primaryKey"`
	UserID string `json:"-" gorm:"index"`
	Text   string `json:"text"`
	Done   bool   `json:"done"`
}

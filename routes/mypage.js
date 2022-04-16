//  /mypage/パスのルーター
const router = require("express").Router();

router.get('/', (req, res, next) => {
  res.render("./account/index.ejs", {title: 'Mypage'});
});

module.exports = router;
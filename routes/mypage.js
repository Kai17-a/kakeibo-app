//  /mypage/パスのルーター
const router = require("express").Router();
const db = require('../models/index.js');

router.get('/', (req, res, next) => {
  db.Users.findAll().then(usrs => {
    var data = {
      title: 'Mypage',
      content: usrs
    }
    res.render("./account/index.ejs", data);
  });
});

module.exports = router;
'use strict';
const {
  Model
} = require('sequelize');
module.exports = (sequelize, DataTypes) => {
  class Users extends Model {
    /**
     * Helper method for defining associations.
     * This method is not a part of Sequelize lifecycle.
     * The `models/index` file will call this method automatically.
     */
    static associate(models) {
      // define association here
    }
  }
  Users.init({
    user_id: {
      type: DataTypes.INTEGER,
      primaryKey: true
    },
    user_name: DataTypes.STRING,
    user_pass: DataTypes.STRING,
    user_login: DataTypes.DATE
  }, {
    sequelize,
    modelName: 'Users',
    freezeTableName: true
  });
  return Users;
};
'use strict';
const {
  Model
} = require('sequelize');
module.exports = (sequelize, DataTypes) => {
  class Details extends Model {
    /**
     * Helper method for defining associations.
     * This method is not a part of Sequelize lifecycle.
     * The `models/index` file will call this method automatically.
     */
    static associate(models) {
      // define association here
    }
  }
  Details.init({
    detail_id: DataTypes.INTEGER,
    category_id: DataTypes.INTEGER,
    payment_id: DataTypes.INTEGER,
    user_id: DataTypes.INTEGER,
    cost: DataTypes.INTEGER,
    date: DataTypes.STRING,
    description: DataTypes.STRING
  }, {
    sequelize,
    modelName: 'Details',
  });
  return Details;
};
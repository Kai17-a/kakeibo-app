SELECT b.amount, c.name, COALESCE(SUM(CAST(e.amount AS INTEGER)), 0)
FROM budgets b
JOIN expense_categories c ON c.id = b.category_id
LEFT JOIN expenses e ON e.category_id = b.category_id AND strftime('%Y-%m', e.transaction_date) = ?2
WHERE b.category_id = ?1
GROUP BY b.amount, c.name;

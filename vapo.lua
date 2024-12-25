local query = vapo.dstr()

function vapo.draw(ui)
  ui:label("Choose your thing")
  ui:input(query)
  ui:label(string.format("Query: %s", query))
end

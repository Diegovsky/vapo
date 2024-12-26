local query = vapo.dstr()

function vapo.draw(ui)
  ui:label("Digite algo:")
  ui:input(query)
  if ui:button("Me aperte") then
    query:set("foda: "..tostring(query))
  end
end

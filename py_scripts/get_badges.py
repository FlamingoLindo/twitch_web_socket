"""
Docstring for get_badges
"""

import json
import re
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait

data = []
driver = webdriver.Firefox()
wait = WebDriverWait(driver, 5)

driver.get('https://www.streamdatabase.com/twitch/global-badges?\
           sort_by=added_at&sort_direction=ascending')


badge_element = wait.until(EC.presence_of_element_located(
    (By.CSS_SELECTOR, 'a.relative')))

badges = driver.find_elements(By.CSS_SELECTOR, "a.relative")

for i, badge in enumerate(badges, 1):
    img = badge.find_element(By.CSS_SELECTOR, "img")
    img_src = img.get_attribute("src") or ""
    img_alt = img.get_attribute("alt") or ""

    img_alt = img_alt.lower()
    img_alt = img_alt.replace(' ', '')
    # img_alt = img_alt.replace('-', '_')
    # img_alt = re.sub(r'[^a-z0-9_]', '', img_alt)
    # img_alt = re.sub(r'_+', '_', img_alt)
    # img_alt = img_alt.strip('_')

    img_src = re.sub(r'/\d+$', '/{SIZE}', img_src)

    # print(i)
    # print(img_alt)
    # print(img_src)

    data.append(
        {'index': i, 'name': img_alt, 'url': img_src})


with open('twitch/json/twitch_badges.json', 'w', encoding='utf-8') as f:
    json.dump(data, f, ensure_ascii=False, indent=4)

driver.quit()

package com.aira.android

import org.junit.Assert.assertEquals
import org.junit.Test

class AttributionTest {
    @Test fun nameAndRepoMatchPermission() {
        assertEquals("Aira for Android", Attribution.APP_NAME)
        assertEquals("https://github.com/gilangggra/Aira", Attribution.REPO_URL)
    }
}

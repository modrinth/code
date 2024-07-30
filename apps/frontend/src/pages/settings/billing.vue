<template>
  <div>
    <button class="btn" @click="beginSetup">start flow test</button>
    <div class="card">
      <div id="payment-element">
        <!-- Elements will create form elements here -->
      </div>
      <div id="error-message">
        <!-- Display error message to your customers here -->
      </div>
      <button id="submit" class="btn" @click="submit">Submit</button>
    </div>
  </div>
  <div class="card"></div>
</template>
<script setup>
useHead({
  script: [
    {
      src: "https://js.stripe.com/v3/",
    },
  ],
});

let stripe = null;
let elements = null;

const { data: paymentMethods, refresh } = await useAsyncData("billing/payment_methods", () =>
  useBaseFetch("billing/payment_methods", { internal: true }),
);

console.log(paymentMethods)

onMounted(() => {
  stripe = Stripe(
    "pk_test_51JbFxJJygY5LJFfKV50mnXzz3YLvBVe2Gd1jn7ljWAkaBlRz3VQdxN9mXcPSrFbSqxwAb0svte9yhnsmm7qHfcWn00R611Ce7b",
  );
});

async function beginSetup() {
  try {
    const result = await useBaseFetch("billing/payment_method", {
      internal: true,
      method: "POST",
    });

    const styles = getComputedStyle(document.body);

    elements = stripe.elements({
      theme: "night",
      clientSecret: result.client_secret,
      appearance: {
        variables: {
          colorPrimary: styles.getPropertyValue("--color-brand"),
          colorBackground: styles.getPropertyValue("--color-bg"),
          colorText: styles.getPropertyValue("--color-base"),
          colorTextPlaceholder: styles.getPropertyValue("--color-secondary"),
          colorDanger: styles.getPropertyValue("--color-red"),
          fontFamily: styles.getPropertyValue("--font-standard"),
          spacingUnit: "0.25rem",
          borderRadius: "1rem",
        },
      },
    });

    const paymentElement = elements.create("payment");
    paymentElement.mount("#payment-element");
  } catch (err) {
    console.error(err);
  }
}

async function submit(event) {
  event.preventDefault();

  const { error } = await stripe.confirmSetup({
    // `Elements` instance that was used to create the Payment Element
    elements,
    confirmParams: {
      return_url: "http://localhost:3000/settings/billing",
    },
  });

  if (error) {
    // This point will only be reached if there is an immediate error when
    // confirming the payment. Show error to your customer (for example, payment
    // details incomplete)
    const messageContainer = document.querySelector("#error-message");
    messageContainer.textContent = error.message;
  } else {
    // Your customer will be redirected to your `return_url`. For some payment
    // methods like iDEAL, your customer will be redirected to an intermediate
    // site first to authorize the payment, then redirected to the `return_url`.
  }
}
</script>
